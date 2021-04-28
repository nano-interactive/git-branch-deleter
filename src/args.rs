use clap::Arg;

pub fn path_to_repository<'a, 'b>() -> Arg<'a, 'b> {
    let arg = Arg::with_name("path")
        .short("p")
        .long("path")
        .number_of_values(1)
        .env("GIT_BRANCH_DELETER_REPO_PATH")
        .aliases(&["repo", "repository", "project"])
        .help("Path to the repository")
        .required(false);

    #[cfg(target_os = "windows")]
    let arg = arg.case_insensitive(false);

    #[cfg(not(target_os = "windows"))]
    let arg = arg.case_insensitive(true);

    arg
}

pub fn skip_branch_arg<'a, 'b>() -> Arg<'a, 'b> {
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
        .env("GIT_BRANCH_DELETER_SKIP_BRANCHES")
        .default_value("master")
        .help("Skip branches for removal")
}

pub fn branch_type<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("filter")
        .short("f")
        .long("filter")
        .case_insensitive(true)
        .allow_hyphen_values(false)
        .required(true)
        .default_value("both")
        .possible_values(&["remote", "local", "both"])
        .help("Filters the branches from the repository")
}

pub fn ssh_key_path<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("ssh_key")
        .short("k")
        .long("ssh-key")
        .case_insensitive(false)
        .allow_hyphen_values(true)
        .help("Path to the SSH key used for authentication to remote git repository")
        .required(false)
        .env("GIT_BRANCH_DELETER_SSH_KEY")
        .aliases(&["key", "ssh"])
}

pub fn ssh_key_passphrase<'a, 'b>() -> Arg<'a, 'b> {
    Arg::with_name("ssh_key_passphrase")
        .short("l")
        .long("ssh-key-passphrase")
        .case_insensitive(true)
        .takes_value(false)
        .allow_hyphen_values(true)
        .help("SSH Key passphrase")
        .required(false)
        .env("GIT_BRANCH_DELETER_SSH_KEY_PASSPHRASE")
}
