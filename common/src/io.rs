use directories::ProjectDirs;
use std::path::{Path, PathBuf};
use std::{env, fs};

pub const QUALIFIER: &str = "dev";
pub const ORGANIZATION: &str = "xairaven";
pub const APPLICATION: &str = "xailyser";

pub fn get_storage_file_path(
    file_name: &str, file_kind: FileKind,
) -> Result<PathBuf, std::io::Error> {
    let dirs = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION);
    match dirs {
        None => {
            let mut current_dir = env::current_dir()?;
            current_dir.push(file_name);
            Ok(current_dir)
        },
        Some(value) => Ok(file_kind.into_path(&value).join(file_name)),
    }
}

pub fn create_parent_directories(path: &Path) -> Result<(), std::io::Error> {
    if let Some(parent_path) = path.parent() {
        return fs::create_dir_all(parent_path);
    }

    Ok(())
}

pub enum FileKind {
    Data,
    Config,
}

impl FileKind {
    pub fn into_path(self, project_dirs: &ProjectDirs) -> &Path {
        match self {
            FileKind::Config => project_dirs.config_dir(),
            FileKind::Data => project_dirs.data_dir(),
        }
    }
}
