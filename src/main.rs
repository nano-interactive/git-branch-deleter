mod args;
mod git;

use args::{branch_type, path_to_repository, skip_branch_arg};
use clap::App;
use crossterm::style::Colorize;
use git::{get_branches, get_git_repo, GitBranch};
use git2::Error as GitError;
use std::{
    fmt::Display,
    io::{stdin, stdout, Read, Result, Stdin, Stdout, Write},
};
use std::{fs::read_dir, path::Path};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
const ABOUT: &str = "Deletes old branches from the GIT repository";

const ACTIONS: &str = "k(eep)/d(elete)/s(how)/q(uit)";

#[derive(Debug)]
enum BranchAction {
    Show,
    Keep,
    Delete,
    Quit,
    Invalid,
}

impl Display for BranchAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BranchAction::Show => f.write_str("Show"),
            BranchAction::Keep => f.write_str("Keep"),
            BranchAction::Delete => f.write_str("Delete"),
            BranchAction::Quit => f.write_str("Quit"),
            BranchAction::Invalid => f.write_str("Invalid"),
        }
    }
}

impl From<u8> for BranchAction {
    fn from(c: u8) -> Self {
        match c {
            b'd' => Self::Delete,
            b'k' => Self::Keep,
            b's' => Self::Show,
            b'q' => Self::Quit,
            _ => Self::Invalid,
        }
    }
}

fn get_action(out: &mut Stdout, input: &mut Stdin) -> Result<BranchAction> {
    input.lock();
    let mut buf: [u8; 3] = [0; 3];
    let action = "Action: ".green();

    write!(out, "{}", action)?;

    out.flush()?;
    input.read(&mut buf)?;

    Ok(BranchAction::from(buf[0].to_ascii_lowercase()))
}

fn print_git_branch_info(stdout: &mut Stdout, branch: &GitBranch) {
    let _ = writeln!(
        stdout,
        "Actions: {}\nBranch -> {}\nLast Commit -> {}",
        ACTIONS,
        branch.get_name().blue(),
        branch.get_commit_time().to_string().yellow(),
    );
}

fn get_public_and_private_key_paths<'a>() -> std::result::Result<Vec<String>, GitError> {
    // TODO: Improve SSH Support for different keys
    let home = std::env::var("HOME").unwrap_or("/root".to_owned());
    // TODO: Add CommandLine flag to ssh-key
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

/// delete-branch function tries to remove branch from the repository
/// Uses all ssh keys found in user's .ssh directory, and tries the one by one.
/// If any of the succeed, operation is considered successful.
/// Furthore optimizations: parallelize the execution of the keys
fn delete_branch(branch: &mut GitBranch, keys: &Vec<String>) -> std::result::Result<(), GitError> {
    let deleted = keys
        .into_iter()
        .map(|key| match branch.delete(&key) {
            Ok(_) => Some(key),
            Err(err) => {
                #[cfg(debug_assertions)]
                dbg!(err);
                None
            }
        })
        .any(|k| k.is_some());

    if deleted {
        Ok(())
    } else {
        Err(GitError::from_str("Branch is not delete"))
    }
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

    for branch in branches {
        do_action_on_branch(&mut stdin, &mut stdout, branch, &keys);
    }

    Ok(())
}

fn do_action_on_branch(
    stdin: &mut Stdin,
    stdout: &mut Stdout,
    mut branch: GitBranch,
    keys: &Vec<String>,
) {
    print_git_branch_info(stdout, &branch);
    while let Ok(action) = get_action(stdout, stdin) {
        match action {
            BranchAction::Delete => match delete_branch(&mut branch, keys) {
                Ok(_) => {
                    println!(
                        "Branch {} was successfully deleted.",
                        branch.get_name().blue()
                    );
                    break;
                }
                Err(err) => {
                    println!(
                        "Error while deleting branch {}: {}",
                        branch.get_name().red(),
                        err
                    );
                    break;
                }
            },
            BranchAction::Keep | BranchAction::Quit => break,
            BranchAction::Show => {
                println!("Commit Message -> {}", branch.get_commit_message().green())
            }
            _ => {
                eprintln!("{} action, valid actions are: {}", action, ACTIONS)
            }
        }
    }
}
