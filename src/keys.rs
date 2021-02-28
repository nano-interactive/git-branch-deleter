use std::{env::var, fs::read_dir, path::Path};

use git2::Error;

pub fn get_public_and_private_key_paths<'a>() -> Result<Vec<String>, Error> {
    // TODO: Improve SSH Support for different keys
    let home = var("HOME").unwrap_or("/root".to_owned());
    // TODO: Add CommandLine flag to ssh-key
    let ssh_key = var("GIT_DELETER_SSH").unwrap_or("".to_owned());
    let ssh_key_path = Path::new(&ssh_key);

    if ssh_key != "" && ssh_key_path.is_relative() {
        return Ok(vec![format!("{}/{}", home, ssh_key)]);
    } else if ssh_key != "" {
        return Ok(vec![ssh_key]);
    }

    let path = format!("{}/.ssh/", home);

    let entries = read_dir(Path::new(&path)).map_err(|err| Error::from_str(&err.to_string()))?;

    let data = entries
        .filter_map(|p| match p {
            Ok(entry) => {
                let name = entry.file_name();
                let name = name.to_str().unwrap();
                let start = &name[..3];
                let end = &name[name.len() - 4..];

                if start == "id_" && end != ".pub" {
                    Some(format!("{}{}", path, name))
                } else {
                    None
                }
            }
            Err(_) => None,
        })
        .collect::<Vec<_>>();

    Ok(data)
}
