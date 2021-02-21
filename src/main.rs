use clap::{App, Arg};
use git2::Repository;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
const ABOUT: &str = "Deletes old branches from the GIT repository";

#[derive(Debug)]
struct Flags<'a> {
    skip: Vec<&'a str>,
    path: Option<&'a str>,
}

fn main() -> Result<(), git2::Error> {
    let matches = App::new(NAME)
        .about(ABOUT)
        .author(AUTHOR)
        .version(VERSION)
        .arg(skip_branch_arg())
        .arg(path_to_repository())
        .get_matches();

    let skip = matches.values_of("skip").unwrap().collect::<Vec<&str>>();
    let path = matches.value_of("path");

    let repo = get_git_repo(path)?;

    let flags = Flags { skip, path };

    dbg!(flags);

    println!("{}", repo.namespace().unwrap());
    Ok(())
}

fn get_git_repo(path: Option<&str>) -> Result<git2::Repository, git2::Error> {
    match path {
        Some(p) => git2::Repository::open(p),
        None => git2::Repository::open_from_env(),
    }
}

fn path_to_repository<'a, 'b>() -> Arg<'a, 'b> {
    let arg = Arg::with_name("path")
        .short("p")
        .long("path")
        .aliases(&["repo", "repository", "project"])
        .help("Path to the repository")
        .required(false);

    #[cfg(target_os = "windows")]
    let arg = arg.case_insensitive(false);

    #[cfg(not(target_os = "windows"))]
    let arg = arg.case_insensitive(true);

    arg
}

fn skip_branch_arg<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("skip")
        .short("s")
        .long("skip")
        .alias("omit")
        .max_values(10)
        .min_values(1)
        .case_insensitive(false)
        .allow_hyphen_values(false)
        .multiple(true)
        .required(true)
        .default_value("master")
        .help("Skip branches for removal")
}
