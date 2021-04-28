use chrono::NaiveDateTime;
use git2::{Branch, Repository, Remote, PushOptions, Cred, Error};
use crate::branch_type::BranchType;
use std::cmp::Ordering;
use std::path::Path;

/// Wrapper for Git Branch type
/// Encapsulates branch name, last commit time, last commit message
/// and branch type (remote or local)
pub struct GitBranch<'a> {
    pub(crate) commit_id: String,
    pub(crate) name: String,
    pub(crate) message: String,
    pub(crate) commit_time: NaiveDateTime,
    pub(crate) branch: Branch<'a>,
    pub(crate) branch_type: BranchType,
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


impl<'a> GitBranch<'a> {
    fn find_remote(
        &self,
        repo: &'a Repository,
        origin_name: &str,
        private_key: &'a str,
        passphrase: Option<&'a str>,
    ) -> Result<(Remote<'a>, PushOptions), Error> {
        let remote = repo.find_remote(origin_name)?;
        let mut remote_callbacks = git2::RemoteCallbacks::new();
        let mut options = PushOptions::default();

        remote_callbacks.credentials(move |_, username, types| {
            let username = username.unwrap_or("git");

            if types.is_ssh_key() || types.is_ssh_memory() {
                let private_key = Path::new(&private_key);

                Cred::ssh_key(username, None, private_key, passphrase)
            } else if types.is_username() {
                Cred::username(username)
            } else {
                Err(Error::from_str("No credentials found"))
            }
        });

        options.remote_callbacks(remote_callbacks);

        Ok((remote, options))
    }

    /// Delete the branch from the repository, if the branch is ```remote```, it will
    /// be removed from all remote origins if the credentials allow
    pub fn delete(&mut self, repo: &'a Repository, private_key: &str, passphrase: Option<&str>) -> Result<(), Error> {
        self.branch.delete()?;

        if self.branch_type == BranchType::Remote {
            let origin_name = self.name.split('/');

            let origin_name = origin_name.collect::<Vec<&str>>();

            let (mut remote, mut options) = self.find_remote(repo, origin_name[0], private_key, passphrase)?;

            remote.push(
                &[format!("+:refs/heads/{}", origin_name[1])],
                Some(&mut options),
            )?
        }

        Ok(())
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

    pub fn get_commit_id(&self) -> &str {
        &self.commit_id
    }
}