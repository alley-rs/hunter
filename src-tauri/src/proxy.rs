#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
#[cfg(target_os = "macos")]
use std::process::exit;
use std::{
    env,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::{self, Command},
};

#[cfg(target_os = "macos")]
use regex::Regex;
use serde::Serialize;
use sysinfo::System;
use tokio::sync::RwLock;
use tracing::{debug, error, info, trace};
#[cfg(target_os = "windows")]
use {
    winapi::um::winbase::CREATE_NO_WINDOW,
    windows_registry::{Key, Value, CURRENT_USER},
};

use crate::{
    config::{AUTOSTART_DIR, CONFIG, CONFIG_DIR, EXECUTABLE_DIR, TROJAN_CONFIG_FILE_PATH},
    error::{Error, HunterResult},
    node::ServerNode,
};

#[cfg(not(target_os = "windows"))]
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

#[cfg(target_os = "linux")]
#[derive(Debug, Clone)]
enum Desktop {
    KDE,
    GNOME,
}

impl Desktop {
    fn read_kde_proxy_config(&self) -> HunterResult<Option<(String, String)>> {
        let proxy_type = execute(
            "kreadconfig5",
            vec![
                "--file",
                "kioslaverc",
                "--group",
                "Proxy Settings",
                "--key",
                "ProxyType",
            ],
            "get proxy type",
        )?;

        trace!("read_proxy_type result: {}", proxy_type);

        if proxy_type != "2" {
            return Ok(None);
        }

        let proxy_config_script = execute(
            "kreadconfig5",
            vec![
                "--file",
                "kioslaverc",
                "--group",
                "Proxy Settings",
                "--key",
                "Proxy Config Script",
            ],
            "get Proxy Config Script",
        )?;

        Ok(Some((proxy_type, proxy_config_script)))
    }

    pub fn read_gnome_proxy_config(&self) -> HunterResult<Option<(String, String)>> {
        let mode = execute(
            "gsettings",
            vec!["get", "org.gnome.system.proxy", "mode"],
            "read proxy config mode",
        )?;

        if mode != "auto" {
            return Ok(None);
        }

        let url = execute(
            "gsettings",
            vec!["get", "org.gnome.system.proxy", "autoconfig-url"],
            "read proxy auto config url",
        )?;

        Ok(Some((mode, url)))
    }

    fn read_proxy_config(&self) -> HunterResult<Option<(String, String)>> {
        match self {
            Desktop::KDE => self.read_kde_proxy_config(),
            Desktop::GNOME => self.read_gnome_proxy_config(),
        }
    }

    fn set_kde_proxy_config(&self, pac: &str) -> HunterResult<()> {
        execute(
            "kwriteconfig5",
            vec![
                "--file",
                "kioslaverc",
                "--group",
                "Proxy Settings",
                "--key",
                "ProxyType",
                "2",
            ],
            "set proxy type",
        )?;
        execute(
            "kwriteconfig5",
            vec![
                "--file",
                "kioslaverc",
                "--group",
                "Proxy Settings",
                "--key",
                "Proxy Config Script",
                pac,
            ],
            "set proxy config script",
        )?;

        Ok(())
    }

    fn set_gnome_proxy_config(&self, pac: &str) -> HunterResult<()> {
        execute(
            "gsettings",
            vec!["set", "org.gnome.system.proxy", "mode", "auto"],
            "set proxy config mode",
        )?;

        execute(
            "gsettings",
            vec!["set", "org.gnome.system.proxy", "autoconfig-url", pac],
            "set proxy auto config url",
        )?;

        Ok(())
    }

    fn set_proxy_config(&self, pac: &str) -> HunterResult<()> {
        match self {
            Desktop::KDE => self.set_kde_proxy_config(pac),
            Desktop::GNOME => self.set_gnome_proxy_config(pac),
        }
    }

    fn disable_kde_auto_proxy(&self) -> HunterResult<()> {
        execute(
            "kwriteconfig5",
            vec![
                "--file",
                "kioslaverc",
                "--group",
                "Proxy Settings",
                "--key",
                "ProxyType",
                "0",
            ],
            "set proxy type",
        )?;

        Ok(())
    }

    fn disable_gnome_auto_proxy(&self) -> HunterResult<()> {
        execute(
            "gsettings",
            vec!["set", "org.gnome.system.proxy", "mode", "none"],
            "set proxy config mode",
        )?;

        Ok(())
    }

    fn disable_auto_proxy(&self) -> HunterResult<()> {
        match self {
            Desktop::KDE => self.disable_kde_auto_proxy(),
            Desktop::GNOME => self.disable_gnome_auto_proxy(),
        }
    }
}

impl TryFrom<&str> for Desktop {
    type Error = String;
    fn try_from(value: &str) -> std::result::Result<Self, String> {
        match value {
            "KDE" => Ok(Desktop::KDE),
            "GNOME" => Ok(Desktop::GNOME),
            _ => Err(format!("unkown desktop: {}", value)),
        }
    }
}

#[cfg(target_os = "linux")]
#[derive(Debug, Clone)]
pub struct Proxy {
    desktop: Desktop,
    pub executable_file: PathBuf,
    auto_start_desktop: PathBuf,
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
        #[cfg(not(target_os = "windows"))]
        let result = String::from_utf8_lossy(&output.stdout);

        return Ok(result.trim().to_owned());
    }

    #[cfg(target_os = "windows")]
    let err = gbk_to_utf8(&output.stderr)?;
    #[cfg(not(target_os = "windows"))]
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
        {
            Ok(Proxy {
                executable_file,
                auto_start_vbs: AUTOSTART_DIR.join("trojan-go.vbs"),
                child_id: 0,
                daemon: false,
            })
        }

        #[cfg(target_os = "linux")]
        {
            let desktop =
                env::var("XDG_SESSION_DESKTOP").map_err(|e| Error::Other(e.to_string()))?;

            Ok(Proxy {
                desktop: Desktop::try_from(desktop.as_str()).map_err(|e| Error::Other(e))?,
                executable_file,
                auto_start_desktop: AUTOSTART_DIR.join("trojan-go.desktop"),
                child_id: 0,
                daemon: false,
            })
        }
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
        debug!("setting pac: {}", pac);

        #[cfg(target_os = "macos")]
        execute(
            "networksetup",
            vec!["-setautoproxyurl", &self.service, pac],
            "配置自动代理脚本",
        )?;

        #[cfg(target_os = "windows")]
        {
            let key = self.get_registry_key(true)?;
            key.set_string("AutoConfigURL", pac).map_err(|e| {
                error!("设置 AutoConfigURL 失败：{}", e);
                e
            })?;
        }

        #[cfg(target_os = "linux")]
        self.desktop.set_proxy_config(pac)?;

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
        {
            let key = self.get_registry_key(true)?;
            key.set_string("AutoConfigURL", "").map_err(|e| {
                error!("取消 AutoConfigURL 失败：{}", e);
                e
            })?;
        }

        #[cfg(target_os = "linux")]
        self.desktop.disable_auto_proxy()?;

        Ok(())
    }

    pub async fn get_trojan_process_state(&mut self) -> HunterResult<Option<TrojanProcessState>> {
        trace!("获取 trojan 进程状态");

        #[derive(Debug)]
        struct ProcessState {
            pid: u32,
            second: String,
            third: String,
        }

        let mut state = ProcessState {
            pid: 0,
            second: String::new(),
            third: String::new(),
        };

        #[cfg(not(target_os = "windows"))]
        let name = "trojan-go";
        #[cfg(target_os = "windows")]
        let name = "trojan-go.exe";

        let sys = System::new_all();
        for p in sys.processes_by_exact_name(name) {
            debug!("获取到进程：{} {:?}", p.pid(), p.cmd());
            state.pid = p.pid().as_u32();
            if p.cmd().len() >= 3 {
                state.second = p.cmd()[1].clone();
                state.third = p.cmd()[2].clone();
            }
            break;
        }

        if state.pid == 0 {
            info!("未检测到 trojan-go 进程");
            return Ok(None);
        }

        info!("检测到 trojan-go 进程：{:?}", state);

        if state.second != "-config" || Path::new(&state.third) != *TROJAN_CONFIG_FILE_PATH {
            info!("检测到不是由本程序创建的 trojan 进程");
            return Ok(Some(TrojanProcessState::Other { pid: state.pid }));
        }

        // trojan 正在运行时，在 trojan config 中获取到的 node 一定是有效的
        let config = CONFIG.read().await;
        let server_node = config.get_using_server_node().await?.cloned();
        drop(config);

        match server_node {
            None => {
                info!("检测到由本程序创建的 trojan 进程，但其配置文件中使用了无效节点");

                Ok(Some(TrojanProcessState::Invalid { pid: state.pid }))
            }
            Some(n) => {
                info!("检测到由本程序创建的 trojan 进程");

                // trojan 进程由本程序启动且在程序运行前存在则为 daemon 进程
                self.child_id = state.pid;
                self.daemon = true;

                Ok(Some(TrojanProcessState::Daemon(n)))
            }
        }
    }

    #[cfg(target_os = "linux")]
    pub fn auto_proxy_url_state(&self) -> HunterResult<bool> {
        let proxy_config = self.desktop.read_proxy_config()?;

        match proxy_config {
            None => Ok(false),
            Some((proxy_type, proxy_config_script)) => {
                debug!(
                    "查询代理状态命令结果：type={} script={}",
                    proxy_type, proxy_config_script
                );

                Ok(true)
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
    fn get_registry_key(&self, write: bool) -> HunterResult<Key> {
        if !write {
            CURRENT_USER
                .open(r#"Software\Microsoft\Windows\CurrentVersion\Internet Settings"#)
                .map_err(Error::Registry)
        } else {
            CURRENT_USER
                .create(r#"Software\Microsoft\Windows\CurrentVersion\Internet Settings"#)
                .map_err(Error::Registry)
        }
    }

    #[cfg(target_os = "windows")]
    pub fn auto_proxy_url_state(&self) -> HunterResult<bool> {
        let key = self.get_registry_key(false)?;

        let auto_config_url = match key.get_value("AutoConfigURL")? {
            Value::String(s) => s,
            _ => unreachable!(),
        };

        debug!("获取到 auto config url: {}", auto_config_url);

        if auto_config_url.len() == 0 {
            info!("自动 pac 未设置");
            return Ok(false);
        }

        info!("自动 pac 已设置：{}", auto_config_url);

        Ok(true)
    }

    pub fn check_executable_file(&self) -> bool {
        self.executable_file.exists()
    }

    fn execute_command(&self) -> HunterResult<Command> {
        trace!("运行 trojan");

        let out_log_file = fs::File::create(CONFIG_DIR.join("hunter-out.log")).map_err(|e| {
            error!("create hunter.log failed: {}", e);
            e
        })?;
        let error_log_file =
            fs::File::create(CONFIG_DIR.join("hunter-error.log")).map_err(|e| {
                error!("create hunter-error.log failed: {}", e);
                e
            })?;

        let mut cmd = new_command(
            &self.executable_file,
            vec!["-config", TROJAN_CONFIG_FILE_PATH.to_str().unwrap()],
            #[cfg(debug_assertions)]
            "使用配置文件运行 trojan",
        );

        cmd.stdout(out_log_file).stderr(error_log_file);

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

    pub fn auto_start_state(&self) -> bool {
        #[cfg(target_os = "macos")]
        if self.auto_start_plist.exists() {
            return true;
        }

        #[cfg(target_os = "windows")]
        if self.auto_start_vbs.exists() {
            return true;
        }

        #[cfg(target_os = "linux")]
        if self.auto_start_desktop.exists() {
            return true;
        }

        false
    }

    #[cfg(target_os = "linux")]
    pub fn switch_auto_start(&self, current_state: bool) -> HunterResult<()> {
        if current_state {
            debug!("delete auto start script");
            fs::remove_file(&self.auto_start_desktop).map_err(|e| {
                error!("delete auto start script failed: {}", e);
                e
            })?;

            info!("auto start script has been deleted");

            return Ok(());
        }

        let content = format!(
            r#"[Desktop Entry]
Exec={} -config {}
Icon=dialog-scripts
Name=trojan-go
Path=
Type=Application
X-KDE-AutostartScript=true
"#,
            self.executable_file.to_string_lossy(),
            TROJAN_CONFIG_FILE_PATH.to_string_lossy()
        );

        debug!("add auto start script");

        fs::write(&self.auto_start_desktop, content).map_err(|e| {
            error!("add auto start script failed: {}", e);
            e
        })?;

        info!("auto start script has been created");

        Ok(())
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

    #[cfg(not(target_os = "windows"))]
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
