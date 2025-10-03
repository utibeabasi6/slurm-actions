use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubEvent {
    pub after: String,
    pub base_ref: Option<String>,
    pub before: Option<String>,
    pub commits: Vec<Commit>,
    pub compare: String,
    pub created: bool,
    pub deleted: bool,
    pub forced: bool,
    pub head_commit: Commit,
    pub installation: Installation,
    pub pusher: CommitPusher,
    #[serde(rename = "ref")]
    pub ref_: String,
    pub repository: Repository,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub added: Vec<String>,
    pub author: CommitAuthor,
    pub display_name: Option<String>,
    pub committer: CommitAuthor,
    pub distinct: bool,
    pub id: String,
    pub message: String,
    pub modified: Vec<String>,
    pub removed: Vec<String>,
    pub timestamp: String,
    pub tree_id: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitAuthor {
    pub email: String,
    pub name: String,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Installation {
    pub id: u64,
    pub node_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitPusher {
    pub email: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub allow_forking: bool,
    pub archive_url: String,
    pub archived: bool,
    pub assignees_url: String,
    pub blobs_url: String,
    pub branches_url: String,
    pub clone_url: String,
    pub collaborators_url: String,
    pub comments_url: String,
    pub commits_url: String,
    pub compare_url: String,
    pub contents_url: String,
    pub contributors_url: String,
    pub created_at: u64,
    pub default_branch: String,
    pub deployments_url: String,
    pub description: Option<String>,
    pub disabled: bool,
    pub full_name: String,
    pub name: String,
    pub master_branch: String,
}
