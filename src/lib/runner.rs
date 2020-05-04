use slog::*;
use std::{thread, time};

use super::docker::DockerStage;
use super::git::GitStage;
use super::kube::KubeStage;
use super::slack::{SlackChatPublish, SlackMessage};
use super::watch::WatchStage;

#[derive(Debug)]
pub struct StageRunner {
    pub id: String,
    pub repo: String,
    pub tag: String,
    pub sha: String,
    pub path: String,
    pub logger: slog::Logger,
    pub slack_channel: crossbeam::Sender<String>,
}

impl StageRunner {

    pub fn new(id: String, repo: String, tag: String, path: String, logger: slog::Logger, slack_channel: crossbeam::Sender<String>) -> StageRunner {
        StageRunner {
            id: id,
            repo: repo,
            tag: tag,
            sha: "".to_string(),
            path: path,
            logger: logger,
            slack_channel: slack_channel,
        }
    }

    //
    // run the deploy stages:
    // git, docker, kube, watch
    //

    pub fn call(&mut self) -> Option<i32> {
        let mut git_stage = GitStage::new(
            &self.id,
            &self.repo,
            &self.tag,
            self.logger.clone(),
        );

        info!(self.logger, "git_stage_starting"; "id" => &self.id);

        self._slack_message("git_stage_starting", "pending");

        match git_stage.call() {
            Some(0) => {
                info!(self.logger, "git_stage_completed"; "id" => &self.id);

                // update git sha
                self.sha = git_stage.sha
            },
            Some(code) => {
                info!(self.logger, "git_stage_exception"; "code" => code, "id" => &self.id);

                self._slack_message("git_stage_exception", "error");

                return Some(code)
            },
            None => {
                info!(self.logger, "git_stage_exception"; "code" => 500, "id" => &self.id);

                self._slack_message("git_stage_exception", "error");

                return Some(500)
            }
        };

        let mut docker_stage = DockerStage::new(
            &self.id,
            &self.path,
            self.logger.clone(),
        );

        info!(self.logger, "docker_stage_starting"; "id" => &self.id);

        self._slack_message("docker_stage_starting", "pending");

        match docker_stage.call() {
            Some(0) => {
                info!(self.logger, "docker_stage_completed"; "id" => &self.id);
            },
            Some(code) => {
                info!(self.logger, "docker_stage_exception"; "code" => code, "id" => &self.id);

                self._slack_message("docker_stage_exception", "error");

                return Some(code)
            },
            None => {
                info!(self.logger, "docker_stage_exception"; "code" => 500, "id" => &self.id);

                self._slack_message("docker_stage_exception", "error");

                return Some(500)
            }
        };

        let kube_stage = KubeStage::new(
            &self.id,
            &self.path,
            &docker_stage.image_tag,
            self.logger.clone(),
        );

        info!(self.logger, "kube_stage_starting"; "id" => &self.id);

        self._slack_message("kube_stage_starting", "pending");

        match kube_stage.call() {
            Some(0) => {
                info!(self.logger, "kube_stage_completed"; "id" => &self.id);

                self._slack_message("kube_stage_completed", "pending");
            },
            Some(code) => {
                info!(self.logger, "kube_stage_exception"; "code" => code, "id" => &self.id);

                self._slack_message("kube_stage_exception", "error");

                return Some(code)
            },
            None => {
                info!(self.logger, "kube_stage_exception"; "code" => 500, "id" => &self.id);

                self._slack_message("kube_stage_exception", "error");

                return Some(500)
            }
        };

        info!(self.logger, "watch_stage_starting"; "id" => &self.id);

        self._slack_message("watch_stage_starting", "pending");

        let watch_stage = WatchStage::new(
            &self.id,
            &self.path,
            self.logger.clone(),
        );

        let watch_objects = match watch_stage.call() {
            Err(_) => {
                return Some(0)
            },
            Ok(objects) => {
                objects
            }
        };

        let mut waited = 0;

        for watch_object in watch_objects.iter() {
            loop {
                match watch_object.call() {
                    Ok(0) => {
                        info!(self.logger, "watch_stage_completed"; "id" => &self.id);

                        self._slack_message("watch_stage_completed", "success");

                        break;
                    },
                    Ok(_) => {
                        self._slack_message("watch_stage_pending", "pending");
                    },
                    Err(e) => {
                        error!(self.logger, "watch_stage_exception: {}", e);

                        self._slack_message("watch_stage_pending", "error");

                        break;
                    }
                };

                thread::sleep(time::Duration::from_secs(watch_object.sleep));

                waited += watch_object.sleep;

                if waited > watch_object.wait {
                    break;
                }
            }
        };

        info!(self.logger, "deploy_completed"; "sha" => &self.sha, "path" => &self.path, "id" => &self.id);

        Some(0)
    }

    fn _slack_message(&self, subject: &str, state: &str) -> Option<i32> {
        let message = SlackMessage {
            subject: subject.to_string(),
            state: state.to_string(),
            id: self.id.to_string(),
            resource: self.path.to_string(),
            git_repo: self.repo.to_string(),
            git_tag: self.tag.to_string(),
            git_sha: self.sha.to_string(),
        };

        match SlackChatPublish::call(&self.slack_channel, &message) {
            Err(e) => {
                info!(self.logger, "slack_message_send_error: {}", e);
            },
            Ok(_) => {}
        }

        Some(0)
    }
}
