use chrono::naive::NaiveDateTime;
use chrono::Duration;
use git2::{Branch, BranchType, Commit, Cred, Error, PushOptions, Remote, Repository};
use std::{cmp::Ordering, path::Path, str::FromStr};

pub struct GitBranch<'a> {
    name: String,
    message: String,
    commit_time: NaiveDateTime,
    branch: Branch<'a>,
    repo: &'a Repository,
    branch_type: BranchType,
}

impl<'a> GitBranch<'a> {
    fn find_remote(
        &self,
        origin_name: &str,
        private_key: &'a str,
    ) -> Result<(Remote<'a>, PushOptions), Error> {
        let remote = self.repo.find_remote(origin_name)?;
        let mut remote_callbacks = git2::RemoteCallbacks::new();
        let mut options = PushOptions::default();

        remote_callbacks.credentials(move |_, username, types| {
            let username = username.unwrap_or("git");

            if types.is_ssh_key() || types.is_ssh_memory() {
                let private_key = Path::new(&private_key);
                Cred::ssh_key(username, None, private_key, None)
            } else if types.is_username() {
                Cred::username(username)
            } else {
                Err(Error::from_str("No credentials found"))
            }
        });

        options.remote_callbacks(remote_callbacks);

        Ok((remote, options))
    }

    pub fn delete(&mut self, private_key: &str) -> Result<(), Error> {
        self.branch.delete()?;

        if self.branch_type == BranchType::Remote {
            let origin_name = self.name.split('/');

            let origin_name = origin_name.collect::<Vec<&str>>();

            let (mut remote, mut options) = self.find_remote(origin_name[0], private_key)?;

            remote.push(
                &[format!("+:refs/heads/{}", origin_name[1])],
                Some(&mut options),
            )?
        }

        Ok(())
    }

    pub fn get_filter(t: &str) -> Result<Option<BranchType>, Error> {
        match t {
            "remote" => Ok(Some(BranchType::Remote)),
            "local" => Ok(Some(BranchType::Local)),
            "both" => Ok(None),
            _ => Err(Error::from_str("Invalid branch filter")),
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_commit_time(&self) -> NaiveDateTime {
        self.commit_time
    }

    pub fn get_commit_message(&self) -> &str {
        &self.message
    }
}

impl<'a> Eq for GitBranch<'a> {
    fn assert_receiver_is_total_eq(&self) {}
}

impl<'a> PartialEq for GitBranch<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl<'a> PartialOrd for GitBranch<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.commit_time.partial_cmp(&other.commit_time)
    }
}

impl<'a> Ord for GitBranch<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.commit_time.cmp(&other.commit_time)
    }
}

fn extract_commit_time(commit: &Commit) -> NaiveDateTime {
    let time = commit.time();
    let seconds = time.seconds();
    let offset = time.offset_minutes();
    let time = NaiveDateTime::from_timestamp(seconds, 0);
    time + Duration::minutes(offset as i64)
}

fn extract_commit_message<'a>(commit: &'a Commit) -> Option<&'a str> {
    commit.message()
}

fn extract_branch_name<'a>(branch: &'a Branch, skip: &'_ Vec<&'_ str>) -> Option<String> {
    match branch.name() {
        Ok(value) => match value {
            Some(name) => {
                if skip.contains(&name) {
                    None
                } else {
                    Some(String::from(name))
                }
            }
            None => None,
        },

        Err(_) => None,
    }
}

/// Open the git repo
pub fn get_git_repo(path: Option<&str>) -> Result<Repository, Error> {
    match path {
        Some(p) => Repository::open(p),
        None => Repository::open_from_env(),
    }
}

/// get_branches retrievs all branches from the repository
/// sorted by the timestamp on there commits
/// it ignores ones with errors
pub fn get_branches<'a>(
    repo: &'a Repository,
    filter: Option<BranchType>,
    skip: &Vec<&str>,
) -> Result<Vec<GitBranch<'a>>, Error> {
    let mut branches = repo
        .branches(filter)?
        .into_iter()
        .filter_map(|branch| -> Option<GitBranch> {
            match branch {
                Ok((branch, branch_type)) => {
                    let name = extract_branch_name(&branch, &skip);
                    let commit = branch.get().peel_to_commit().unwrap();
                    let commit_time = extract_commit_time(&commit);
                    let message = extract_commit_message(&commit);
                    if let Some(name) = name {
                        Some(GitBranch {
                            name,
                            commit_time,
                            message: String::from_str(message.unwrap()).unwrap(),
                            branch,
                            repo,
                            branch_type,
                        })
                    } else {
                        None
                    }
                }
                Err(_) => None,
            }
        })
        .collect::<Vec<GitBranch>>();

    branches.sort_unstable();
    Ok(branches)
}
