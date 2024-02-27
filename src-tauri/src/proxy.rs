#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
#[cfg(target_os = "macos")]
use std::process::exit;
use std::{
    ffi::OsStr,
    fs,
    path::PathBuf,
    process::{self, Command},
};

use regex::Regex;
use serde::Serialize;
use tokio::sync::RwLock;
#[cfg(target_os = "windows")]
use winapi::um::winbase::CREATE_NO_WINDOW;

use crate::{
    config::{AUTOSTART_DIR, CONFIG, CONFIG_DIR, EXECUTABLE_DIR, TROJAN_CONFIG_FILE_PATH},
    error::{Error, HunterResult},
    node::ServerNode,
};

#[cfg(target_os = "macos")]
pub const EXECUTABLE_FILE: &str = "trojan-go";
#[cfg(target_os = "windows")]
pub const EXECUTABLE_FILE: &str = "trojan-go.exe";

lazy_static! {
    pub static ref PROXY: RwLock<Proxy> = RwLock::new(new_proxy());
}

fn new_proxy() -> Proxy {
    match Proxy::new() {
        Ok(p) => p,
        Err(e) => {
            error!("初始化 proxy 错误：{}", e);
            process::exit(1);
        }
    }
}

#[cfg(target_os = "macos")]
#[derive(Debug, Clone)]
pub struct Proxy {
    service: String,
    pub executable_file: PathBuf,
    auto_start_plist: PathBuf,
    child_id: u32,
    daemon: bool,
}

#[cfg(target_os = "windows")]
#[derive(Clone, Debug)]
pub struct Proxy {
    pub executable_file: PathBuf,
    auto_start_vbs: PathBuf,
    child_id: u32,
    daemon: bool,
}

#[cfg(target_os = "windows")]
fn gbk_to_utf8(bytes: &[u8]) -> HunterResult<String> {
    use encoding::all::GBK;
    use encoding::{DecoderTrap, Encoding};

    let decoded = GBK
        .decode(bytes, DecoderTrap::Strict)
        .map_err(|e| Error::Other(e.to_string()))?;

    Ok(decoded)
}

fn new_command<C, A, AS>(cmd: C, args: AS, #[cfg(debug_assertions)] log_desc: &str) -> Command
where
    C: AsRef<OsStr>,
    A: AsRef<OsStr>,
    AS: IntoIterator<Item = A>,
{
    let mut cmd = Command::new(cmd);

    cmd.args(args);

    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);

    #[cfg(debug_assertions)]
    {
        debug!("命令({})：{:?}", log_desc, cmd);
    }

    cmd
}

fn execute<C, A, AS>(cmd: C, args: AS, log_desc: &str) -> HunterResult<String>
where
    C: AsRef<OsStr>,
    A: AsRef<OsStr>,
    AS: IntoIterator<Item = A>,
{
    let mut cmd = new_command(
        cmd,
        args,
        #[cfg(debug_assertions)]
        log_desc,
    );

    let output = cmd.output()?;

    if output.status.success() {
        #[cfg(target_os = "windows")]
        let result = gbk_to_utf8(&output.stdout)?;
        #[cfg(target_os = "macos")]
        let result = String::from_utf8_lossy(&output.stdout);

        return Ok(result.trim().to_owned());
    }

    #[cfg(target_os = "windows")]
    let err = gbk_to_utf8(&output.stderr)?;
    #[cfg(target_os = "macos")]
    let err = String::from_utf8_lossy(&output.stderr);

    error!("命令({})执行失败：{:?} -> {}", log_desc, cmd, err);

    Err(Error::Command(err.to_string()))
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "UPPERCASE")]
pub enum TrojanProcessState {
    Daemon(ServerNode),
    Invalid { pid: u32 },
    Other { pid: u32 },
}

impl Proxy {
    pub fn new() -> HunterResult<Self> {
        if !EXECUTABLE_DIR.exists() {
            info!("hunter 缓存目录不存在, 自动创建: {:?}", *EXECUTABLE_DIR);
            fs::create_dir(EXECUTABLE_DIR.as_path())?;
        }

        if !CONFIG_DIR.exists() {
            info!("hunter 配置目录不存在, 自动创建: {:?}", *CONFIG_DIR);
            fs::create_dir(CONFIG_DIR.as_path())?;
        }
        let executable_file = EXECUTABLE_DIR.join(EXECUTABLE_FILE);

        #[cfg(target_os = "macos")]
        {
            let service = get_active_network_service()?;

            match service {
                Some(s) => Ok(Proxy {
                    service: s,
                    executable_file,
                    auto_start_plist: AUTOSTART_DIR.join("com.thepoy.hunter.plist"),
                    child_id: 0,
                    daemon: false,
                }),
                None => {
                    error!("未能找到正在使用的网络服务，退出程序");
                    exit(1);
                }
            }
        }

        #[cfg(target_os = "windows")]
        Ok(Proxy {
            executable_file,
            auto_start_vbs: AUTOSTART_DIR.join("trojan-go.vbs"),
            child_id: 0,
            daemon: false,
        })
    }

    pub fn child_id(&self) -> u32 {
        self.child_id
    }

    pub fn reset_child_id(&mut self) {
        self.child_id = 0;
    }

    pub fn daemon(&self) -> bool {
        self.daemon
    }

    pub fn switch_daemon(&mut self) {
        self.daemon = !self.daemon;

        info!("后台驻留已{}", if self.daemon { "开启" } else { "关闭" });
    }

    pub async fn set_server_node(&mut self, name: &str) -> HunterResult<()> {
        if self.child_id > 0 {
            kill(self.child_id)?;
        }

        let config = CONFIG.read().await;
        config.set_server_node(name).await?;

        self.execute()
    }

    pub fn enable_auto_proxy_url(&self, pac: &str) -> HunterResult<()> {
        #[cfg(target_os = "macos")]
        execute(
            "networksetup",
            vec!["-setautoproxyurl", &self.service, pac],
            "配置自动代理脚本",
        )?;

        #[cfg(target_os = "windows")]
        execute(
            "reg",
            vec![
                "add",
                r#""HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Internet Settings""#,
                "/v",
                "AutoConfigURL",
                "/t",
                "REG_SZ",
                "/d",
                &format!(r#""{pac}""#),
                "/f",
            ],
            "配置 pac 注册表",
        )?;

        Ok(())
    }

    pub fn disable_auto_proxy_url(&self) -> HunterResult<()> {
        #[cfg(target_os = "macos")]
        execute(
            "networksetup",
            vec!["-setautoproxystate", &self.service, "off"],
            "关闭使用自动代理脚本",
        )?;

        #[cfg(target_os = "windows")]
        execute(
            "reg",
            vec![
                "add",
                r#""HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Internet Settings""#,
                "/v",
                "AutoConfigURL",
                "/t",
                "REG_SZ",
                "/d",
                r#""""#,
                "/f",
            ],
            "清空 pac 注册表",
        )?;

        Ok(())
    }

    pub async fn get_trojan_process_state(&mut self) -> HunterResult<Option<TrojanProcessState>> {
        trace!("获取 trojan 进程状态");
        #[cfg(target_os = "macos")]
        let output = execute(
            "sh",
            vec![
                "-c",
                r#"ps -ef | grep trojan | grep -v grep | awk '{print $2, $9, $10, $11}'"#,
            ],
            "查询 trojan 进程信息",
        )
        .map_err(|e| {
            error!("获取 trojan 进程失败：{}", e);
            e
        })?;

        #[cfg(target_os = "windows")]
        let output = execute(
            "powershell",
            vec![
                "Get-WmiObject",
                "Win32_Process",
                "-Filter",
                r#""name = 'trojan-go.exe'""#,
                "|",
                "Select-Object",
                "-Property",
                "CommandLine,ProcessId",
                "|",
                "Format-List",
                "|",
                "Out-String",
                "-Width",
                "4096",
            ],
            "查询 trojan 进程信息",
        )?;

        if output.len() == 0 {
            debug!("未查询到 trojan 进程");
            return Ok(None);
        }

        debug!("获取到 trojan 进程信息：'{}'", output);

        #[cfg(target_os = "macos")]
        let params = output.splitn(2, " ").map(|s| s.to_string()).collect();

        #[cfg(target_os = "windows")]
        let params = {
            let params: Vec<String> = output.split(" ").map(|s| s.to_string()).collect();

            let mut fourth = params[4].to_string().replace("\\", "/");
            if fourth.contains("\r\n") {
                fourth = fourth.splitn(2, "\r\n").collect::<Vec<&str>>()[0].to_string();
            }

            vec![
                params[params.len() - 1].to_string(),
                format!("{} {}", params[3].to_string(), fourth),
            ]
        };

        info!("有效进程信息：{:?}", params);

        let pid_string = &params[0];

        debug!("trojan 进程 id：{:?}", pid_string);

        let pid: u32 = pid_string.parse()?;

        #[cfg(target_os = "windows")]
        let params_should_be = format!(
            "-config {}",
            TROJAN_CONFIG_FILE_PATH.to_str().unwrap().replace("\\", "/")
        );
        #[cfg(not(target_os = "windows"))]
        let params_should_be = format!("-config {}", TROJAN_CONFIG_FILE_PATH.to_str().unwrap());

        debug!("params_should_be: {}", params_should_be);

        if params[1] != params_should_be {
            info!("检测到不是由本程序创建的 trojan 进程");
            return Ok(Some(TrojanProcessState::Other { pid }));
        }

        // trojan 正在运行时，在 trojan config 中获取到的 node 一定是有效的
        let config = CONFIG.read().await;
        let server_node = config.get_using_server_node().await?.cloned();
        drop(config);

        match server_node {
            None => {
                info!("检测到由本程序创建的 trojan 进程，但其配置文件中使用了无效节点");

                Ok(Some(TrojanProcessState::Invalid { pid }))
            }
            Some(n) => {
                info!("检测到由本程序创建的 trojan 进程");

                // trojan 进程由本程序启动且在程序运行前存在则为 daemon 进程
                self.child_id = pid;
                self.daemon = true;

                Ok(Some(TrojanProcessState::Daemon(n)))
            }
        }
    }

    #[cfg(target_os = "macos")]
    pub fn auto_proxy_url_state(&self) -> HunterResult<bool> {
        let output = execute(
            "networksetup",
            vec!["-getautoproxyurl", &self.service],
            "查询设置中的代理状态",
        )?;

        debug!("查询代理状态命令结果：\n{}", output);

        let re = Regex::new(r#"\nEnabled: (.+)"#)?;
        let caps = re.captures(&output);

        if let Some(caps) = caps {
            let state = caps.get(1).map(|m| m.as_str());

            if let Some(state) = state {
                debug!("代理开启状态：{}", state);
                return Ok(state != "No");
            }
        }

        debug!("系统代理未配置");

        Ok(false)
    }

    #[cfg(target_os = "windows")]
    pub fn auto_proxy_url_state(&self) -> HunterResult<bool> {
        let output = execute(
            "reg",
            vec![
                "query",
                r#"HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Internet Settings"#,
                "/v",
                "AutoConfigURL",
            ],
            "查询 pac 注册表",
        )?;

        if output.len() == 0 {
            return Ok(false);
        }

        let re = Regex::new(r#"AutoConfigURL    REG_SZ    (.+)"#)?;
        let caps = re.captures(&output);

        if let Some(caps) = caps {
            let state = caps.get(1).map(|m| m.as_str());

            debug!("pac 配置：{:?}", state);

            if let Some(state) = state {
                debug!("系统代理已启用");
                return Ok(state != "No");
            }
        }

        debug!("系统代理未启用");
        Ok(false)
    }
    pub fn check_executable_file(&self) -> bool {
        self.executable_file.exists()
    }

    fn execute_command(&self) -> HunterResult<Command> {
        trace!("运行 trojan");

        let log_file = fs::File::open(CONFIG_DIR.join("hunter.log"))?;

        let mut cmd = new_command(
            &self.executable_file,
            vec!["-config", TROJAN_CONFIG_FILE_PATH.to_str().unwrap()],
            #[cfg(debug_assertions)]
            "使用配置文件运行 trojan",
        );

        cmd.stdout(log_file.try_clone().unwrap()).stderr(log_file);

        Ok(cmd)
    }

    pub fn execute(&mut self) -> HunterResult<()> {
        let mut cmd = self.execute_command()?;

        let child = cmd.spawn()?;

        self.child_id = child.id();

        // 默认开启后台驻留
        self.daemon = true;

        info!("已创建 trojan 子进程：{}", self.child_id);

        Ok(())
    }

    #[cfg(target_os = "macos")]
    pub fn auto_start_state(&self) -> bool {
        if self.auto_start_plist.exists() {
            return true;
        }

        false
    }

    #[cfg(target_os = "windows")]
    pub fn auto_start_state(&self) -> bool {
        if self.auto_start_vbs.exists() {
            return true;
        }

        false
    }

    #[cfg(target_os = "windows")]
    pub fn switch_auto_start(&self, current_state: bool) -> HunterResult<()> {
        if current_state {
            debug!("删除开机启动脚本");
            return Ok(fs::remove_file(&self.auto_start_vbs)?);
        }

        let content = format!(
            r#"set ws=WScript.CreateObject("WScript.Shell")
ws.Run "{} -config {}",0"#,
            self.executable_file.to_string_lossy(),
            TROJAN_CONFIG_FILE_PATH.to_str().unwrap()
        );

        debug!("添加开机启动脚本");
        Ok(fs::write(&self.auto_start_vbs, content)?)
    }

    #[cfg(target_os = "macos")]
    pub fn switch_auto_start(&self, current_state: bool) -> HunterResult<()> {
        if current_state {
            debug!("删除开机启动脚本");
            return Ok(fs::remove_file(&self.auto_start_plist)?);
        }

        let content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple Computer//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
   <key>Label</key>
   <string>com.hunter.trojan-go</string>
   <key>ProgramArguments</key>
   <array>
     <string>{}</string>
     <string>-config</string>
     <string>{}</string>
    </array>
   <key>RunAtLoad</key>
   <true/>
</dict>
</plist>
"#,
            self.executable_file.to_string_lossy(),
            TROJAN_CONFIG_FILE_PATH.to_str().unwrap()
        );

        debug!("添加开机启动脚本");
        Ok(fs::write(&self.auto_start_plist, content)?)
    }
}

pub fn kill(id: u32) -> HunterResult<()> {
    if id == 0 {
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    execute("kill", vec!["-9", &id.to_string()], "杀死进程")?;

    #[cfg(target_os = "windows")]
    execute("taskkill", vec!["/F", "/PID", &id.to_string()], "杀死进程")?;

    Ok(())
}

#[cfg(target_os = "macos")]
fn get_active_network_service() -> HunterResult<Option<String>> {
    trace!("获取正在使用的网络服务");
    // 使用网络配置命令获取当前正在使用的网络服务的信息
    let output = execute(
        "networksetup",
        vec!["-listallnetworkservices"],
        "查看网络服务列表",
    )?;

    debug!("网络服务列表：\n{}", output);

    // 解析命令输出，提取当前正在使用的网络服务
    for part in output[60..].split("\n") {
        let mut chars = part.chars();
        if chars.next() == Some('*') {
            continue;
        }

        let router = get_service_info(part)?;

        if router.is_some() {
            debug!("正在使用的网络服务：{}", part);
            return Ok(Some(part.to_string()));
        }
    }

    Ok(None)
}

#[cfg(target_os = "macos")]
fn get_service_info(device: &str) -> HunterResult<Option<String>> {
    // 使用网络配置命令获取指定网络服务的详细信息
    let output = execute("networksetup", vec!["-getinfo", device], "查看网络信息")?;

    debug!("{} 网络服务信息：\n{}", device, output);

    let re = Regex::new(r#"\nRouter: (.+)"#)?;
    for caps in re.captures_iter(&output) {
        let router = caps.get(1).map(|m| m.as_str()).unwrap();

        if router != "none" {
            return Ok(Some(router.to_string()));
        }
    }

    Ok(None)
}
