mod actions;
mod args;
mod git;
mod keys;

use crate::actions::get_action;
use crate::keys::get_public_and_private_key_paths;
use actions::BranchAction;
use args::{branch_type, path_to_repository, skip_branch_arg};
use clap::App;
use crossterm::style::Colorize;
use git::{get_branches, get_git_repo, BranchType, GitBranch};
use git2::{Error as GitError, Repository};
use std::io::{stdin, stdout, Stdin, Stdout, Write};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
const ABOUT: &str = "Deletes old branches from the GIT repository";

const ACTIONS: &str = "k(eep)/d(elete)/s(how)/q(uit)";

fn print_git_branch_info(stdout: &mut Stdout, branch: &GitBranch) {
    let _ = writeln!(
        stdout,
        "Actions: {}\nBranch -> {}\nLast Commit -> {}",
        ACTIONS,
        branch.get_name().blue(),
        branch.get_commit_time().to_string().yellow(),
    );
}

/// delete-branch function tries to remove branch from the repository
/// Uses all ssh keys found in user's .ssh directory, and tries the one by one.
/// If any of the succeed, operation is considered successful.
/// Furthore optimizations: parallelize the execution of the keys
fn delete_branch<'a>(
    repo: &'a Repository,
    branch: &mut GitBranch<'a>,
    keys: &Vec<String>,
) -> Result<(), GitError> {
    let deleted = keys
        .into_iter()
        .map(|key| match branch.delete(repo, key) {
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
) {
    print_git_branch_info(stdout, &branch);
    while let Ok(action) = get_action(stdout, stdin) {
        match action {
            BranchAction::Delete => match delete_branch(repo, &mut branch, keys) {
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

fn main() -> Result<(), GitError> {
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
    let branch_filter = BranchType::from(matches.value_of("filter").unwrap());

    if branch_filter == BranchType::Invalid {
        return Err(GitError::from_str("Invalid branch filter"));
    }

    let keys = get_public_and_private_key_paths()?;
    let repo = get_git_repo(path)?;
    let branches = get_branches(&repo, branch_filter, &skip)?;

    for branch in branches {
        do_action_on_branch(&mut stdin, &mut stdout, &repo, branch, &keys);
    }

    Ok(())
}

