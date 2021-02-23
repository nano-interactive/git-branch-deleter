use clap::Arg;

pub fn path_to_repository<'a, 'b>() -> Arg<'a, 'b> {
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
