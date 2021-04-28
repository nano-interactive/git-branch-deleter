mod actions;
mod args;
mod branch;
mod branch_type;
mod git;
mod keys;

use crate::actions::BranchAction;
use crate::actions::{get_action, get_ssh_key_passphrase};
use crate::args::{
    branch_type, path_to_repository, skip_branch_arg, ssh_key_passphrase, ssh_key_path,
};
use crate::branch::GitBranch;
use crate::branch_type::BranchType;
use crate::keys::{get_public_and_private_key_paths, DefaultPathsExtractor};
use clap::App;
use crossterm::style::Colorize;
use git::{get_branches, get_git_repo};
use git2::{Error as GitError, Repository};
use std::io::{stdin, stdout, Stdin, Stdout, Write};
use std::rc::Rc;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
const ABOUT: &str = "Deletes old branches from the GIT repository";

const ACTIONS: &str = "k(eep)/d(elete)/s(how)/q(uit)";

/// delete-branch function tries to remove branch from the repository
/// Uses all ssh keys found in user's .ssh directory, and tries the one by one.
/// If any of the succeed, operation is considered successful.
fn delete_branch<'a>(
    repo: &'a Repository,
    branch: &mut GitBranch<'a>,
    keys: &Vec<String>,
    passphrase: Option<&'a str>,
) -> Result<(), GitError> {
    let deleted = keys
        .into_iter()
        .map(|key| match branch.delete(repo, key, passphrase) {
            Ok(_) => Some(key),
            Err(_err) => {
                #[cfg(debug_assertions)]
                dbg!(_err);
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

fn do_action_on_branch<'a>(
    stdin: &mut Stdin,
    stdout: &mut Stdout,
    repo: &'a Repository,
    mut branch: GitBranch<'a>,
    keys: &Vec<String>,
    passphrase: Option<&'a str>,
) {
    let _ = writeln!(
        stdout,
        "Actions: {}\nBranch -> {}\nLast Commit -> {}\nCommit Hash: {}",
        ACTIONS,
        branch.get_name().blue(),
        branch
            .get_commit_time()
            .format("%a %d %B %Y %T")
            .to_string()
            .yellow(),
        branch.get_commit_id()[..10].green(),
    );
    while let Ok(action) = get_action(stdout, stdin) {
        match action {
            BranchAction::Delete => match delete_branch(repo, &mut branch, keys, passphrase) {
                Ok(_) => {
                    print!(
                        "Branch {} was successfully deleted.\n\n",
                        branch.get_name().blue()
                    );
                    break;
                }
                Err(err) => {
                    println!(
                        "Error while deleting branch {}: {}\n\n",
                        branch.get_name().red(),
                        err
                    );
                    break;
                }
            },
            BranchAction::Keep | BranchAction::Quit => break,
            BranchAction::Show => {
                println!(
                    "Commit Hash -> {} | Commit Message -> {}",
                    branch.get_commit_id(),
                    branch.get_commit_message().green()
                )
            }
            _ => {
                eprintln!("{} action, valid actions are: {}", action, ACTIONS)
            }
        }
    }
}

fn main() -> Result<(), GitError> {
    let matches = App::new(NAME)
        .about(ABOUT)
        .author(AUTHOR)
        .version(VERSION)
        .arg(skip_branch_arg())
        .arg(path_to_repository())
        .arg(ssh_key_path())
        .arg(branch_type())
        .arg(ssh_key_passphrase())
        .get_matches();

    let mut stdin = stdin();
    let mut stdout = stdout();

    let skip = matches.values_of("skip").unwrap().collect::<Vec<&str>>();
    let path = matches.value_of("path");
    let ssh_key = matches.value_of("ssh_key");
    let branch_filter = BranchType::from(matches.value_of("filter").unwrap());

    if branch_filter == BranchType::Invalid {
        return Err(GitError::from_str("Invalid branch filter"));
    }

    let mut passphrase: Rc<Option<String>> = Rc::from(None);

    if matches.is_present("ssh_key_passphrase") {
        passphrase = match get_ssh_key_passphrase(&mut stdout) {
            Ok(value) => Rc::from(Some(value)),
            Err(_) => Rc::from(None),
        }
    }

    let keys = get_public_and_private_key_paths(DefaultPathsExtractor::new(), ssh_key)?;
    let repo = get_git_repo(path)?;
    let branches = get_branches(&repo, branch_filter, &skip)?;

    for branch in branches {
        let c = passphrase.clone();
        let pass = match c.as_deref() {
            Some(p) => Some(p),
            None => None,
        };

        do_action_on_branch(&mut stdin, &mut stdout, &repo, branch, &keys, c.as_deref());
    }

    Ok(())
}
