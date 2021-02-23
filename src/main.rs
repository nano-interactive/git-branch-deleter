mod args;
mod git;

use args::{branch_type, path_to_repository, skip_branch_arg};
use clap::App;
use crossterm::style::Colorize;
use git::{get_branch_filter, get_branches, get_git_repo};
use git2::Error as GitError;
use std::io::{stdin, stdout, Read, Result, Stdin, Stdout, Write};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
const ABOUT: &str = "Deletes old branches from the GIT repository";

enum BranchAction {
    Show,
    Keep,
    Delete,
}

fn get_action(input: &mut Stdin) -> Result<Option<BranchAction>> {
    input.lock();
    let enter_key = char::from(13);
    let mut buf: [u8; 1] = [0; 1];

    // input.
    input.read(&mut buf)?;

    let action = match char::from(buf[0].to_ascii_lowercase()) {
        'd' => Some(BranchAction::Delete),
        'k' => Some(BranchAction::Keep),
        's' => Some(BranchAction::Show),
        c if c == enter_key => Some(BranchAction::Keep),
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
    let branch_filter = get_branch_filter(matches.value_of("filter").unwrap())?;

    let repo = get_git_repo(path)?;
    let branches = get_branches(&repo, branch_filter, &skip)?;

    for mut b in branches {
        print_git_branch_info(&mut stdout, &b);

        loop {
            let action = get_action(&mut stdin);
            match action {
                Ok(None) => {}
                Ok(Some(action)) => match action {
                    BranchAction::Delete => {
                        b.delete()?;
                        // repo.remote_delete(name);
                        println!("Branch {} deleted", b.get_name().blue());
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
