use crate::branch_type::BranchType;
use crate::branch::GitBranch;
use chrono::naive::NaiveDateTime;
use chrono::Duration;
use git2::{Branch, Commit, Error, Repository};
use std::str::FromStr;


fn extract_commit_time(commit: &Commit) -> NaiveDateTime {
    let time = commit.time();
    let seconds = time.seconds();
    let offset = time.offset_minutes();
    let time = NaiveDateTime::from_timestamp(seconds, 0);
    time + Duration::minutes(offset as i64)
}

fn extract_branch_name<'a>(branch: &'a Branch, skip: &'_ Vec<&'_ str>) -> Option<String> {
    match branch.name() {
        Ok(Some(value)) => if skip.contains(&value) {
            None
        } else {
            Some(String::from(value))
        },
        Ok(None) => None,
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

/// get_branches retrieves all branches from the repository
/// sorted by the timestamp on there commits
/// it ignores ones with errors
pub fn get_branches<'a>(
    repo: &'a Repository,
    filter: BranchType,
    skip: &Vec<&str>,
) -> Result<Vec<GitBranch<'a>>, Error> {
    let mut branches = repo
        .branches(filter.into())?
        .into_iter()
        .filter_map(|branch| -> Option<GitBranch> {
            match branch {
                Ok((branch, branch_type)) => {
                    let name = extract_branch_name(&branch, &skip);
                    let commit = branch.get().peel_to_commit().unwrap();
                    let commit_time = extract_commit_time(&commit);
                    let message = commit.message();
                    let commit_id = commit.id().to_string();
                    if let Some(name) = name {
                        Some(GitBranch {
                            name,
                            commit_time,
                            commit_id,
                            message: String::from_str(message.unwrap()).unwrap(),
                            branch,
                            branch_type: BranchType::from(branch_type),
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
