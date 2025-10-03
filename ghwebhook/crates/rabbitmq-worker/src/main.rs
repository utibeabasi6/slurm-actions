use crate::services::create_rabbitmq_consumer;
use futures_util::stream::StreamExt;
use glob::glob;
use lib::types::{githubevent::GithubEvent, workflow::GithubWorkflow};
use tempdir::TempDir;
use tokio::{fs, io::AsyncReadExt};

mod config;
mod services;

#[tokio::main]
async fn main() -> Result<(), lib::errors::AppError> {
    dotenvy::dotenv().ok();

    let config = config::AppConfig::new()?;

    let mut consumer = create_rabbitmq_consumer(&config, "ghwebhook", 5).await?;

    let handle = consumer.handle();

    let reqwest_client = reqwest::Client::new();

    let task = tokio::spawn(async move {
        while let Some(delivery) = consumer.next().await {
            let d = match delivery {
                Ok(delivery) => delivery,
                Err(err) => {
                    eprintln!("Failed to consume message: {}", err);
                    continue;
                }
            };
            let data = match d.message().data() {
                Some(data) => data,
                None => {
                    eprintln!("Empty payload, Skipping.");
                    continue;
                }
            };
            let data = match String::from_utf8(data.to_vec()) {
                Ok(data) => data,
                Err(err) => {
                    eprintln!("Failed to parse message data: {}", err);
                    continue;
                }
            };

            let github_event: GithubEvent = match serde_json::from_str(&data) {
                Ok(github_event) => github_event,
                Err(err) => {
                    eprintln!("Failed to parse message data: {}", err);
                    continue;
                }
            };

            let tempdir = match TempDir::new("ghwebhook") {
                Ok(tempdir) => tempdir,
                Err(err) => {
                    eprintln!("Failed to create tempdir: {}", err);
                    continue;
                }
            };

            println!("Cloning git repo: {}", github_event.repository.clone_url);

            let git_repo = match lib::clone_git_repo(
                github_event.repository.clone_url.as_str(),
                tempdir.path(),
            ) {
                Ok(git_repo) => git_repo,
                Err(err) => {
                    eprintln!("Failed to clone git repo: {}", err);
                    continue;
                }
            };

            let temp_dir_str = match tempdir.path().to_str() {
                Some(temp_dir_str) => temp_dir_str,
                None => {
                    eprintln!("Failed to convert tempdir path to string");
                    continue;
                }
            };
            let workflow_files = glob(format!("{}/.github/workflows/*", temp_dir_str).as_str());

            match workflow_files {
                Ok(workflow_files) => {
                    let mut workflows: Vec<lib::types::workflow::GithubWorkflow> = Vec::new();

                    for workflow_file in workflow_files {
                        if let Err(err) = workflow_file {
                            eprintln!("Failed to read workflow file: {}", err);
                            continue;
                        }
                        let workflow_file = workflow_file.unwrap(); // safe to unwrap since we have checked for error
                        let workflow_file_str = match workflow_file.to_str() {
                            Some(workflow_file_str) => {
                                match fs::File::open(workflow_file_str).await {
                                    Ok(mut file) => {
                                        let mut buffer = String::new();
                                        match file.read_to_string(&mut buffer).await {
                                            Ok(_) => buffer,
                                            Err(err) => {
                                                eprintln!("Failed to read workflow file: {}", err);
                                                continue;
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        eprintln!("Failed to open workflow file: {}", err);
                                        continue;
                                    }
                                }
                            }
                            None => {
                                eprintln!("Failed to convert workflow file path to string");
                                continue;
                            }
                        };

                        let parsed_workflow: lib::types::workflow::GithubWorkflow =
                            match serde_yaml::from_str(&workflow_file_str) {
                                Ok(workflow) => workflow,
                                Err(err) => {
                                    eprintln!("Failed to parse workflow from file: {}", err);
                                    continue;
                                }
                            };
                        workflows.push(parsed_workflow);
                    }

                    if workflows.is_empty() {
                        eprintln!("No workflow files found");
                    }

                    let workflows_to_run = workflows
                        .iter()
                        .cloned()
                        .filter(|workflow| {
                            lib::should_trigger_workflow(workflow.clone(), &github_event)
                        })
                        .collect::<Vec<GithubWorkflow>>();

                    if workflows_to_run.is_empty() {
                        eprintln!("No workflows to run");
                        continue;
                    }

                    let repo_name = github_event.repository.name;
                    let repo_full_name = github_event.repository.full_name;
                    let repo_ref = github_event.ref_;
                    let github_token = &config.github_token;

                    'entrypoint: for workflow in workflows_to_run {
                        let third_party_actions = workflow
                            .jobs
                            .iter()
                            .flat_map(|job| {
                                job.1.steps.iter().filter_map(|step| {
                                    if let Some(uses) = &step.uses {
                                        let repo = match uses.split("@").next() {
                                            // ignore the ref for now
                                            Some(repo) => repo,
                                            None => return None,
                                        };
                                        Some(format!("\"{}\"", repo))
                                    } else {
                                        None
                                    }
                                })
                            })
                            .collect::<Vec<String>>();

                        let third_party_actions = third_party_actions.join(" ");

                        for (job_name, job) in workflow.jobs {
                            let runs_on = job.runs_on.clone();
                            let mut base_script = format!(
                                r#"#!/bin/bash
#SBATCH --job-name={job_name}
#SBATCH --ntasks=1
#SBATCH --partition={runs_on}
#SBATCH --nodes=1
#SBATCH --output={repo_name}_{job_name}_%j.log
#SBATCH --error={repo_name}_{job_name}_%j.err

set -e

export WORK_DIR="/tmp/{repo_name}_{job_name}_${{SLURM_JOB_ID}}"
export REPOS=({third_party_actions})
export ACTIONS_CACHE_DIR=/tmp/{repo_name}_{job_name}/actions_cache
export NUM_TASKS=${{#REPOS[@]}}
export REPOS_STR="${{REPOS[*]}}"

srun [ -d $WORK_DIR ] || mkdir -p $WORK_DIR

cleanup() {{
    local exit_code=$?
    echo ""
    echo "Cleanup"
    cd /
    rm -rf $WORK_DIR
    rm -rf $ACTIONS_CACHE_DIR

    echo ""
    echo "=========================================="
    echo "Workflow completed at: $(date)"
    echo "=========================================="
    exit $exit_code
}}

trap cleanup EXIT

echo "Setting up third party actions"

srun [ -d $ACTIONS_CACHE_DIR ] || mkdir -p $ACTIONS_CACHE_DIR

# Setup third party actions
srun --ntasks=1  bash -c '
    cd "$ACTIONS_CACHE_DIR"
    IFS=" " read -r -a REPOS <<< "$REPOS_STR"

    for i in "${{REPOS[@]}}"; do
        echo "Setting up third party action: $i"

        [ -d $i ] || mkdir -p $i
        pushd $i

        git clone "https://github.com/$i" .

        echo "Setup third party action: $i"

        popd
    done
'

"#
                            );
                            println!("Running job {}", job_name);

                            for step in job.steps {
                                let mut step_command = String::from(format!(
                                    "\necho \"Running step: {}\"\n",
                                    step.name.unwrap_or("".to_string()) // TODO: set a default step name
                                ));
                                if let Some(uses) = step.uses {
                                    let repo = match uses.split("@").next() {
                                        // ignore the ref for now
                                        Some(repo) => repo,
                                        None => {
                                            eprintln!("Error running workflow: Invalid step");
                                            // skip this workflow if any step is invalid
                                            continue 'entrypoint;
                                        }
                                    };
                                    if let Some(with) = step.with {
                                        let inputs_array: Vec<String> = with
                                            .iter()
                                            .map(|(key, value)| format!("INPUT_{key}={value}"))
                                            .collect();

                                        let inputs_str = inputs_array.join(",");

                                        step_command.push_str(format!("srun --chdir=$WORK_DIR --export=GITHUB_WORKSPACE=$WORK_DIR,GITHUB_REPOSITORY={repo_full_name},GITHUB_REF={repo_ref},GITHUB_TOKEN={github_token},INPUT_TOKEN={github_token},PATH=\"/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/snap/bin\",RUNNER_TEMP=/tmp,{inputs_str}  /usr/bin/node /tmp/{repo_name}_{job_name}/actions_cache/{repo}/dist/index.js\n").as_str());
                                    } else {
                                        step_command.push_str(format!("srun --chdir=$WORK_DIR --export=GITHUB_WORKSPACE=$WORK_DIR,GITHUB_REPOSITORY={repo_full_name},GITHUB_REF={repo_ref},GITHUB_TOKEN={github_token},INPUT_TOKEN={github_token},PATH=\"/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/snap/bin\",RUNNER_TEMP=/tmp  /usr/bin/node /tmp/{repo_name}_{job_name}/actions_cache/{repo}/dist/index.js\n").as_str());
                                    }
                                } else if let Some(run) = step.run {
                                    step_command.push_str(format!("srun --chdir=$WORK_DIR --export=GITHUB_WORKSPACE=$WORK_DIR,GITHUB_REPOSITORY={repo_full_name},GITHUB_REF={repo_ref},GITHUB_TOKEN={github_token},INPUT_TOKEN={github_token},PATH=\"/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/snap/bin\",RUNNER_TEMP=/tmp  bash -c \"{run}\"\n").as_str()); // we need to manually set INPUT_TOKEN={github_token} else checkout action fails
                                }

                                base_script.push_str(step_command.as_str());
                            }

                            let request_body = serde_json::json!({
                                "job": {
                                    "script": base_script,
                                    "environment": ["PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/snap/bin"],
                                    "current_working_directory": "/home/slurm"
                                }
                            });

                            match reqwest_client
                                .post(format!(
                                    "http://{}:{}/slurm/v0.0.39/job/submit",
                                    config.slurmrestd_host, config.slurmrestd_port
                                ))
                                .header("Content-Type", "application/json")
                                .header("X-SLURM-USER-NAME", config.slurmrestd_user.clone())
                                .header("X-SLURM-USER-TOKEN", config.slurmrestd_token.clone())
                                .json(&request_body)
                                .send()
                                .await
                            {
                                Ok(res) => match res.error_for_status() {
                                    Ok(ok_res) => {
                                        let status = ok_res.status();
                                        match ok_res.text().await {
                                            Ok(text) => {
                                                println!("✅ Success (status {status}): {text}");
                                            }
                                            Err(e) => {
                                                eprintln!("⚠️ Failed to read body: {e}");
                                                continue;
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("⚠️ Slurm returned error status: {e}");
                                        continue;
                                    }
                                },
                                Err(e) => {
                                    eprintln!("Failed to submit job: {}", e);
                                    continue;
                                }
                            };
                        }
                    }
                }
                Err(err) => {
                    eprintln!("Failed to glob workflow files: {}", err);
                }
            }
        }
    });

    task.await
        .map_err(|err| lib::errors::AppError::RabbitMQConsumerConsumeError(err.to_string()))?;

    handle
        .close()
        .await
        .map_err(|err| lib::errors::AppError::RabbitMQConsumerCloseError(err))?;
    println!("consumer closed successfully");
    Ok(())
}
