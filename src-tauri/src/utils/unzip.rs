use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::{
    fs::{create_dir_all, remove_file, File},
    io,
    path::PathBuf,
};
use tracing::{debug, trace};
use zip::ZipArchive;

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
    pub fn extract(&self, remove: bool) {
        let target_dir = match &self.target_dir {
            Some(d) => {
                debug!("使用传入的 target_dir: {}", d.display());
                &d
            }
            None => {
                let parent = self.file_path.parent().unwrap();
                debug!(
                    "使用 file_path 的父目录作为 target_dir: {}",
                    parent.display()
                );
                parent
            }
        };

        let file = File::open(&self.file_path).unwrap();

        let mut archive = ZipArchive::new(file).unwrap();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
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

            trace!("dir: {:?} outpath: {:?}", target_dir, outpath);

            {
                let comment = file.comment();
                if !comment.is_empty() {
                    trace!("File {i} comment: {comment}");
                }
            }

            if (*file.name()).ends_with('/') {
                trace!("File {} extracted to \"{}\"", i, outpath.display());
                create_dir_all(&outpath).unwrap();
            } else {
                trace!(
                    "File {} extracted to \"{}\" ({} bytes)",
                    i,
                    outpath.display(),
                    file.size()
                );
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        create_dir_all(p).unwrap();
                    }
                }
                let mut outfile = File::create(&outpath).unwrap();
                io::copy(&mut file, &mut outfile).unwrap();
            }

            // Get and Set permissions
            #[cfg(unix)]
            {
                use std::fs::set_permissions;
                use std::os::unix::prelude::PermissionsExt;

                if let Some(mode) = file.unix_mode() {
                    set_permissions(&outpath, PermissionsExt::from_mode(mode)).unwrap();
                }
            }
        }

        if remove {
            remove_file(&self.file_path).unwrap();
        }
    }
}
