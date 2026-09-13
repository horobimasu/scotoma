use std::fs;
use std::sync::OnceLock;
use std::path::PathBuf;

use crate::state::PROJECT_ROOT_DIR_PATH;

const DATABASE_FILE_EXT: &str = "scdb"; // scotoma database btw

static BUILD_DATA_DIR_PATH: OnceLock<PathBuf> = OnceLock::new();

pub fn create_db(name: &str) -> PathBuf {
    if PROJECT_ROOT_DIR_PATH.get().is_none() {
        panic!("attempted to call create_db before scotoma root directory was set");
    }

    let builddata_dir_path = init_builddata();

    builddata_dir_path
        .join(format!("{}.{}", name, DATABASE_FILE_EXT))
}

fn init_builddata() -> PathBuf {
    BUILD_DATA_DIR_PATH.get_or_init(|| {
        let builddata_dir_path = PROJECT_ROOT_DIR_PATH
            .get()
            .unwrap()
            .join(".builddata");

        fs::create_dir_all(&builddata_dir_path)
            .unwrap();

        builddata_dir_path
    }).clone()
}
