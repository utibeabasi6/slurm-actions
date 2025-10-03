use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct GithubWorkflow {
    pub name: Option<String>,
    pub on: GithubWorkflowTrigger,
    pub jobs: HashMap<String, GithubWorkflowJob>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct GithubWorkflowTrigger {
    pub push: Option<GithubWorkflowPushTrigger>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct GithubWorkflowPushTrigger {
    pub branches: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct GithubWorkflowJob {
    pub name: Option<String>,
    #[serde(rename = "runs-on")]
    pub runs_on: String,
    pub steps: Vec<GithubWorkflowJobStep>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct GithubWorkflowJobStep {
    pub name: Option<String>,
    pub run: Option<String>,
    pub uses: Option<String>,
    pub with: Option<HashMap<String, String>>,
    pub env: Option<HashMap<String, String>>,
}
