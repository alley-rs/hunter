#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::{ffi::OsStr, process::Command};

#[cfg(debug_assertions)]
use tracing::debug;
use tracing::error;
#[cfg(target_os = "windows")]
use winapi::um::winbase::CREATE_NO_WINDOW;

use crate::error::{Error, HunterResult};

pub fn new_command<C, A, AS>(cmd: C, args: AS, #[cfg(debug_assertions)] log_desc: &str) -> Command
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

#[cfg(target_os = "windows")]
fn gbk_to_utf8(bytes: &[u8]) -> HunterResult<String> {
    use encoding::all::GBK;
    use encoding::{DecoderTrap, Encoding};

    let decoded = GBK
        .decode(bytes, DecoderTrap::Strict)
        .map_err(|e| Error::Other(e.to_string()))?;

    Ok(decoded)
}

pub fn execute<C, A, AS>(cmd: C, args: AS, log_desc: &str) -> HunterResult<String>
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
