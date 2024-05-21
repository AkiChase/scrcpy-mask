use anyhow::{anyhow, Ok, Result};
use std::path::PathBuf;

pub enum ResourceName {
    ScrcpyServer,
    DefaultKeyConfig,
}

pub struct ResHelper {
    pub res_dir: PathBuf,
}

impl ResHelper {
    pub fn res_init(res_dir: &PathBuf) -> Result<()> {
        let res = [ResourceName::ScrcpyServer, ResourceName::DefaultKeyConfig];

        for name in res {
            let file_path = ResHelper::get_file_path(res_dir, name);
            if !file_path.exists() {
                return Err(anyhow!(format!(
                    "Resource missing! {}",
                    file_path.to_str().unwrap()
                )));
            }
        }

        Ok(())
    }
    pub fn get_file_path(dir: &PathBuf, file_name: ResourceName) -> PathBuf {
        match file_name {
            ResourceName::ScrcpyServer => dir.join("scrcpy-mask-server-v2.4"),
            ResourceName::DefaultKeyConfig => dir.join("default-key-config.json"),
        }
    }

    pub fn get_scrcpy_version() -> String {
        String::from("2.4")
    }
}
