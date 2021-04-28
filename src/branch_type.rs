use git2::BranchType as GitBranchType;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BranchType {
    Remote,
    Local,
    Both,
    Invalid,
}

impl From<&str> for BranchType {
    fn from(t: &str) -> Self {
        match t {
            "remote" => Self::Remote,
            "local" => Self::Local,
            "both" => Self::Both,
            _ => Self::Invalid,
        }
    }
}

impl From<GitBranchType> for BranchType {
    fn from(t: GitBranchType) -> Self {
        match t {
            GitBranchType::Remote => Self::Remote,
            GitBranchType::Local => Self::Local,
        }
    }
}

impl Into<Option<GitBranchType>> for BranchType {
    fn into(self) -> Option<GitBranchType> {
        match self {
            BranchType::Remote => Some(GitBranchType::Remote),
            BranchType::Local => Some(GitBranchType::Local),
            _ => None,
        }
    }
}