use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::{
    fs::{create_dir_all, remove_file, File},
    io,
    path::PathBuf,
};
use tracing::{debug, error, trace, warn};
use zip::ZipArchive;

use crate::config::EXECUTABLE_DIR;
use crate::error::{Error, HunterResult};

#[derive(Debug, Deserialize, Serialize)]
pub struct Zip {
    file_path: PathBuf,
    target_dir: Option<PathBuf>,
    extract_files: Vec<String>,
}

impl Display for Zip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Zip(file_path: {}, target_dir: {:?}, extract_files: {:?})",
            self.file_path.display(),
            self.target_dir,
            self.extract_files
        )
    }
}

impl Zip {
    pub fn extract(&self, remove: bool) -> HunterResult<()> {
        let target_dir = match &self.target_dir {
            Some(d) => {
                debug!(message = "使用传入的 target_dir", dir = ?d);
                &d
            }
            None => {
                let parent = match self.file_path.parent() {
                    Some(p) => p,
                    None => {
                        warn!(
                            message = "用户选择的保存目录不存在，使用默认缓存目录",
                            default_dir = ?*EXECUTABLE_DIR
                        );

                        &EXECUTABLE_DIR
                    }
                };
                debug!(
                    message = "使用 file_path 的父目录作为 target_dir",
                   dir = ?parent
                );
                parent
            }
        };

        let file = File::open(&self.file_path).map_err(|e| {
            error!(message = "打开下载文件失败", error = ?e, file = ?self.file_path);
            e
        })?;

        let mut archive = ZipArchive::new(file).map_err(|e| {
            error!(message = "创建 zip 失败", error = ?e);
            Error::Other(e.to_string())
        })?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| {
                error!(message = "获取压缩包的文件失败",index = i, error = ?e);
                Error::Other(e.to_string())
            })?;

            let mut outpath = match file.enclosed_name() {
                Some(path) => path.to_owned(),
                None => continue,
            };

            if self.extract_files.len() > 0
                && !self
                    .extract_files
                    .contains(&outpath.to_string_lossy().to_string())
            {
                continue;
            }

            outpath = target_dir.join(outpath);

            debug!(dir = ?target_dir, outpath = ?outpath);

            {
                let comment = file.comment();
                if !comment.is_empty() {
                    trace!("File {i} comment: {comment}");
                }
            }

            if (*file.name()).ends_with('/') {
                trace!("File {} extracted to \"{}\"", i, outpath.display());
                create_dir_all(&outpath).map_err(|e| {
                    error!(message = "创建目录失败", error = ?e, path = ?outpath);
                    e
                })?;
            } else {
                trace!(
                    "File {} extracted to \"{}\" ({} bytes)",
                    i,
                    outpath.display(),
                    file.size()
                );
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        create_dir_all(p).map_err(|e| {
                            error!(message = "创建目录失败", error = ?e, path = ?p);
                            e
                        })?;
                    }
                }
                let mut outfile = File::create(&outpath).map_err(|e| {
                    error!(message = "创建文件失败", error = ?e, path = ?outpath);
                    e
                })?;

                io::copy(&mut file, &mut outfile).map_err(|e| {
                    error!(message = "复制文件数据失败", error = ?e);
                    e
                })?;
            }

            // Get and Set permissions
            #[cfg(unix)]
            {
                use std::fs::set_permissions;
                use std::os::unix::prelude::PermissionsExt;

                if let Some(mode) = file.unix_mode() {
                    set_permissions(&outpath, PermissionsExt::from_mode(mode)).map_err(|e| {
                        error!(message = "设置文件权限失败", error = ?e);
                        e
                    })?;
                }
            }
        }

        if remove {
            remove_file(&self.file_path).map_err(|e| {
                error!(message = "删除压缩包失败", error = ?e, file_path = ?self.file_path);
                e
            })?;
        }

        Ok(())
    }
}
