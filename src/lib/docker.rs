use super::fs::FsRoot;
use super::kube_resource::KubeResourceParser;

use slog::{error, info};
use std::io::Result;
use std::process::{Command, ExitStatus};

#[derive(Debug)]
pub struct DockerStage {
    pub image_tag: String,
    pub id: String,
    pub resource_file: String,
    pub resource_key: String,
    pub logger: slog::Logger,
}

impl DockerStage {
    pub fn new(id: &String, resource_path: &String, logger: slog::Logger) -> DockerStage {
        let resource_vec: Vec<_> = resource_path.split(":").collect();

        DockerStage {
            image_tag: "".to_owned(),
            id: id.to_owned(),
            resource_file: resource_vec[0].to_owned(),
            resource_key: resource_vec[1].to_owned(),
            logger: logger,
        }
    }

    pub fn call(&mut self) -> Option<i32> {
        let toml_file = format!(
            "{}/{}",
            FsRoot::call(&self.id),
            &self.resource_file,
        );

        let resource_parser = KubeResourceParser::new(
            &toml_file,
            &self.resource_key,
        );

        let resource = match resource_parser.call() {
            Some(value) => {
                value
            },
            None => {
                return Some(400)
            }
        };

        let docker_file = match resource.get("docker_file") {
            Some(s) => {
                s.as_str().unwrap().to_string()  // remove string quotes
            },
            None => {
                return Some(400)
            }
        };

        let image_name = match resource.get("image_name") {
            Some(s) => {
                s.as_str().unwrap().to_string() // remove string quotes
            },
            None => {
                return Some(400)
            }
        };

        self.image_tag = format!("{}:{}", image_name, self.id);

        let docker_uri = dotenv::var("DOCKER_HOST_URI").unwrap();

        match self._docker_build(&docker_uri, &docker_file, &self.image_tag) {
            Ok(status) => {
                if status.success() {
                    info!(self.logger, "docker_build_ok"; "tag" => &self.image_tag);
                } else {
                    error!(self.logger, "docker_build_exception: {}", status);

                    return Some(500)
                }
            },
            Err(e) => {
                error!(self.logger, "docker_build_exception: {}", e);

                return Some(500)
            }
        };

        match self._docker_push(&docker_uri, &self.image_tag) {
            Ok(status) => {
                if status.success() {
                    info!(self.logger, "docker_push_ok"; "tag" => &self.image_tag);
                } else {
                    error!(self.logger, "docker_push_exception: {}", status);

                    return Some(500)
                }
            },
            Err(e) => {
                error!(self.logger, "docker_push_exception: {}", e);

                return Some(500)
            }
        };

        Some(0)
    }

    fn _docker_build(&self, docker_uri: &String, docker_file: &String, image_tag: &String) -> Result<ExitStatus> {
        let status = Command::new("docker")
            .args(&["--host", &docker_uri, "build", "-f", &docker_file, "-t", &image_tag, "."])
            .current_dir(FsRoot::call(&self.id))
            .status()?;

        Ok(status)
    }

    fn _docker_push(&self, docker_uri: &String, image_tag: &String) -> Result<ExitStatus> {
        let status = Command::new("docker")
            .args(&["--host", &docker_uri, "push", &image_tag])
            .current_dir(FsRoot::call(&self.id))
            .status()?;

        Ok(status)
    }

}
