use std::{env::var, fs::read_dir, path::Path};

#[cfg(test)]
use mockall::automock;

use git2::Error;

#[cfg_attr(test, automock)]
pub trait ExtractPaths {
    fn extract_ssh_key_path(&self, path: Option<&str>) -> SshKey;
    fn extract_home_dir_path<'a>(&self) -> String;
}

pub struct DefaultPathsExtractor;

impl DefaultPathsExtractor {
    pub fn new() -> Self {
        Self
    }
}

impl ExtractPaths for DefaultPathsExtractor {
    fn extract_ssh_key_path(&self, path: Option<&str>) -> SshKey {
        match path {
            Some(path) => {
                let os_path = Path::new(path);

                let path = if os_path.is_relative() {
                    format!("{}{}", self.extract_home_dir_path(), path)
                } else {
                    String::from(path)
                };

                if os_path.is_file() {
                    SshKey::File(path)
                } else {
                    SshKey::Dir(path)
                }
            }
            None => SshKey::Dir(format!("{}/.ssh/", self.extract_home_dir_path())),
        }
    }
    fn extract_home_dir_path<'a>(&self) -> String {
        #[cfg(target_os = "windows")]
            let home = format!(
            "{}{}",
            var("SYSTEMDRIVE").unwrap_or("C:".to_owned()),
            var("HOMEPATH").unwrap_or("\\Users\\System".to_owned())
        );

        #[cfg(not(target_os = "windows"))]
            let home = var("HOME").unwrap_or("/root".to_owned());

        home
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SshKey {
    File(String),
    Dir(String),
}

impl SshKey {
    fn get_path(&self) -> &str {
        match self {
            SshKey::File(ref path) | SshKey::Dir(ref path) => path,
        }
    }
}

pub fn get_public_and_private_key_paths<E: ExtractPaths>(
    extractor: E,
    ssh_key: Option<&str>,
) -> Result<Vec<String>, Error> {
    let path = extractor.extract_ssh_key_path(ssh_key);
    if let SshKey::File(path) = path {
        return Ok(vec![path]);
    }

    let entries = read_dir(Path::new(path.get_path()))
        .map_err(|err| Error::from_str(&err.to_string()))?;

    let data = entries
        .filter_map(|p| match p {
            Ok(entry) => {
                let name = entry.file_name();
                let name = name.to_str().unwrap();
                let start = &name[..3];
                let end = &name[name.len() - 4..];

                if start == "id_" && end != ".pub" {
                    Some(format!("{}{}", path.get_path(), name))
                } else {
                    None
                }
            }
            Err(_) => None,
        })
        .collect::<Vec<_>>();

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn home_dir() -> String {
        #[cfg(target_os = "windows")]
            let real_home = format!(
            "{}{}",
            var("SYSTEMDRIVE").unwrap_or("C:".to_owned()),
            var("HOMEPATH").unwrap_or("\\Users\\System".to_owned())
        );

        #[cfg(not(target_os = "windows"))]
            let real_home = var("HOME").unwrap_or("/root".to_owned());

        real_home
    }

    #[test]
    fn test_home_paths() {
        let paths = DefaultPathsExtractor::new();

        let home = paths.extract_home_dir_path();

        assert_eq!(home, home_dir());
    }

    #[test]
    fn test_ssh_dir_when_no_path_is_provided() {
        let paths = DefaultPathsExtractor::new();

        let ssh_path = paths.extract_ssh_key_path(None);

        let real_ssh_path = format!("{}/.ssh/", home_dir());
        assert_eq!(ssh_path, SshKey::Dir(real_ssh_path));
    }
}
