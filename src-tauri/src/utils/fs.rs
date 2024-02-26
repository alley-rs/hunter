use std::{fs, path::PathBuf};

use crate::error::HunterResult;

pub fn create_dir_if_not_exists(paths: &[&PathBuf]) -> HunterResult<()> {
    for path in paths.iter() {
        if path.exists() {
            continue;
        }

        fs::create_dir(path)?;
    }

    Ok(())
}
