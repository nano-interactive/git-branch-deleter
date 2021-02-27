mod args;
mod git;

use args::{branch_type, path_to_repository, skip_branch_arg};
use clap::App;
use crossterm::style::Colorize;
use git::{get_branches, get_git_repo, GitBranch};
use git2::Error as GitError;
use std::io::{stdin, stdout, Read, Result, Stdin, Stdout, Write};
use std::{fs::read_dir, path::Path};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
const ABOUT: &str = "Deletes old branches from the GIT repository";

enum BranchAction {
    Show,
    Keep,
    Delete,
}

fn get_action(out: &mut Stdout, input: &mut Stdin) -> Result<Option<BranchAction>> {
    input.lock();
    let mut buf: [u8; 2] = [0; 2];

    // input.
    write!(out, "Action: ")?;
    out.flush()?;
    input.read(&mut buf)?;

    let action = match char::from(buf[0].to_ascii_lowercase()) {
        'd' => Some(BranchAction::Delete),
        'k' => Some(BranchAction::Keep),
        's' => Some(BranchAction::Show),
        // c if c == enter_key => Some(BranchAction::Keep),
        _ => None,
    };

    Ok(action)
}

fn print_git_branch_info(stdout: &mut Stdout, branch: &git::GitBranch) {
    let _ = writeln!(
        stdout,
        "Actions: k(eep)/d(elete)/s(how)\nBranch -> {}\nLast Commit -> {}",
        branch.get_name().blue(),
        branch.get_commit_time().to_string().yellow(),
    );
}

fn get_public_and_private_key_paths<'a>() -> std::result::Result<Vec<String>, GitError> {
    let home = std::env::var("HOME").unwrap_or("/root".to_owned());
    let ssh_key = std::env::var("GIT_DELETER_SSH").unwrap_or("".to_owned());
    let ssh_key_path = Path::new(&ssh_key);

    if ssh_key != "" && ssh_key_path.is_relative() {
        return Ok(vec![format!("{}/{}", home, ssh_key)]);
    } else if ssh_key != "" {
        return Ok(vec![ssh_key]);
    }

    let path = format!("{}/.ssh/", home);

    let entries =
        read_dir(Path::new(&path)).map_err(|err| git2::Error::from_str(&err.to_string()))?;

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

fn main() -> std::result::Result<(), GitError> {
    let matches = App::new(NAME)
        .about(ABOUT)
        .author(AUTHOR)
        .version(VERSION)
        .arg(skip_branch_arg())
        .arg(path_to_repository())
        .arg(branch_type())
        .get_matches();

    let mut stdin = stdin();
    let mut stdout = stdout();

    let skip = matches.values_of("skip").unwrap().collect::<Vec<&str>>();
    let path = matches.value_of("path");
    let branch_filter = GitBranch::get_filter(matches.value_of("filter").unwrap())?;

    let keys = get_public_and_private_key_paths()?;
    let repo = get_git_repo(path)?;
    let branches = get_branches(&repo, branch_filter, &skip)?;

    for mut b in branches {
        print_git_branch_info(&mut stdout, &b);

        loop {
            let action = get_action(&mut stdout, &mut stdin);
            match action {
                Ok(None) => {
                    eprintln!("Invalid action, please check again");
                }
                Ok(Some(action)) => match action {
                    BranchAction::Delete => {
                        let keys = &keys;
                        let deleted = keys
                            .into_iter()
                            .map(|key| match b.delete(key) {
                                Ok(_) => Some(key),
                                Err(err) => {
                                    eprintln!("{}", err);
                                    None
                                }
                            })
                            .any(|k| k.is_some());

                        if deleted {
                            println!("Branch {} deleted", b.get_name().blue());
                        } else {
                            println!("Branch {} is not deleted", b.get_name().red());
                        }

                        break;
                    }
                    BranchAction::Keep => break,
                    BranchAction::Show => {
                        println!("Commit Message -> {}", b.get_commit_message().green())
                    }
                },
                Err(err) => {
                    eprintln!("Error in input: {}", err);
                    break;
                }
            }
        }
    }
    Ok(())
}
