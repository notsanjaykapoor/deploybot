use serde::{Deserialize, Serialize};
use slog::*;

use super::runner::StageRunner;

use crate::lib::fs::{FsRemove, FsTouch};

#[derive(Debug, Serialize, Deserialize)]
pub struct DeployMessage {
    pub id: String,
    pub repo: String,
    pub tag: String,
    pub path: String,
}

#[derive(Debug)]
pub struct DeployThread {
    pub deploy_channel: crossbeam_channel::Receiver<String>,  // receiver channel
    pub slack_channel: crossbeam_channel::Sender<String>,  // slack channel
    pub logger: slog::Logger,
}

impl DeployThread {

    pub fn new(deploy_channel: crossbeam_channel::Receiver<String>, slack_channel: crossbeam_channel::Sender<String>, logger: slog::Logger) -> DeployThread {
        DeployThread {
            deploy_channel: deploy_channel,
            slack_channel: slack_channel,
            logger: logger,
        }
    }

    pub fn call(&self) {
        info!(self.logger, "deploy_thread_starting");

        loop {
            let data = match self.deploy_channel.recv() {
                Err(e) => {
                    error!(self.logger, "deploy_thread_exception: {}", e);

                    return ()
                },
                Ok(data) => {
                    data
                }
            };

            // parse the json string into a DeployMessage object
            let message: DeployMessage = match serde_json::from_str(&data) {
                Err(e) => {
                    error!(self.logger, "deploy_thread_exception: {}", e);

                    continue
                },
                Ok(object) => {
                    object
                }
            };

            FsTouch::call(&message.id);

            let mut runner = StageRunner::new(
                message.id.clone(),
                message.repo,
                message.tag,
                message.path,
                self.logger.clone(),
                self.slack_channel.clone(),
            );

            let _code = match runner.call() {
                Some(code) => {
                    code
                },
                None => {
                    500
                }
            };

            FsRemove::call(&message.id);
        }
    }
}
