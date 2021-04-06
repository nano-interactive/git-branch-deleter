# git-branch-deleter

> Developer: Dusan Malusev <dmalusev@nanointeractive.com>


## About

In my current company i was working on some legacy project, which had a lot of old git branches, many of them must be deleted from the repository. This branches 
are just garbage, they were merged a long time ago, but there is not easy way to know how old were this branches, when was the last commit on them, etc.
I sat down one night and came up with this tool, ```git-branch-deleter```. It displays all remote and local branches and asks what to do with the currently displayed branch. (Keep, Remove, Show Commit message, etc.)


## Installation


## Usage

### Arguments

- ```--path```  (aliases ``` repo, repository, project ```) Path to the repository where this actions will be performed. On unix systems paths are case sensitive. (Environmental Variable ```GIT_BRANCH_DELETER_REPO_PATH```)
- ```--skip``` (aliases ``` omit ```) Branches to skip - defaults to `master` (Environmental Variable ```GIT_BRANCH_DELETER_SKIP_BRANCHES```)
- ```--filter``` Filters the branches based on the locality ``` local, remote, both ```. defaults to `both`.
- ```--ssh-key``` (aliases ``` key, ssh ```) Path to the SSH key used for authentication to remote git repository. (Environmental Variable ```GIT_BRANCH_DELETER_SSH_KEY```)

### Examples

```sh
# Execute on the current project
$ git-branch-deleter # No arguments needed

# Use the ssh key to remove only remote branches
$ git-branch-deleter --ssh-key /home/username/.ssh/id_ed25519 --filter remote --path /path/to/project

# Skip the branches in the output
$ git-branch-deleter --ssh-key /home/username/.ssh/id_ed25519 --filter remote --skip origin/master,develop,feat/some-other-branch
```
