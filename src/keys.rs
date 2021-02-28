use std::{env::var, fs::read_dir, path::Path};

use git2::Error;


enum SshKey {
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

fn extract_home_dir_path<'a>() -> String {
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

fn extract_ssh_key_path<'a>(path: Option<&'a str>) -> SshKey {
    match path {
        Some(path) => {
            let os_path = Path::new(path);

            let path = if os_path.is_relative() {
                format!("{}{}", extract_home_dir_path(), path)
            } else {
                String::from(path)
            };

            if os_path.is_file() {
                SshKey::File(path)
            } else {
                SshKey::Dir(path)
            }

        },
        None => SshKey::Dir(format!("{}/.ssh/",  extract_home_dir_path()))
    }
}

pub fn get_public_and_private_key_paths(ssh_key: Option<&str>) -> Result<Vec<String>, Error> {
    let path = extract_ssh_key_path(ssh_key);

    if let SshKey::File(path) = path {
        return Ok(vec![path]);
    }

    let entries = read_dir(Path::new(path.get_path())).map_err(|err| Error::from_str(&err.to_string()))?;

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
