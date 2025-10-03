use std::path::Path;

use crate::types::{githubevent::GithubEvent, workflow::GithubWorkflow};

pub mod errors;
pub mod types;

pub fn clone_git_repo(repo_url: &str, dest: &Path) -> Result<git2::Repository, errors::AppError> {
    let repo = git2::Repository::clone(repo_url, dest)
        .map_err(|err| errors::AppError::GitCloneError(err.to_string()))?;

    Ok(repo)
}

pub fn should_trigger_workflow(workflow: GithubWorkflow, github_event: &GithubEvent) -> bool {
    let ref_split = github_event.ref_.split('/').collect::<Vec<&str>>();
    if ref_split[1] == "heads" {
        if let Some(push_trigger) = workflow.on.push {
            if push_trigger.branches.contains(&ref_split[2].to_string()) {
                return true;
            }
        }
    }

    return false;
}
