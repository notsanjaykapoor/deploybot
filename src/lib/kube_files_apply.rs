use super::fs::FsRoot;
use super::kube_resource::KubeResourceParser;

use slog::{info, error};
use std::io::Result;
use std::process::{Command, ExitStatus};

#[derive(Debug)]
pub struct KubeFilesApply {
    pub id: String,
    pub resource_file: String,
    pub resource_key: String,
    pub files: Vec<String>,
}

impl KubeFilesApply {

    pub fn new(id: &String, resource_file: &String, resource_key: &String, files: Vec<String>) -> KubeFilesApply {
        KubeFilesApply {
            id: id.to_owned(),
            resource_file: resource_file.to_owned(),
            resource_key: resource_key.to_owned(),
            files: files,
        }
    }

    pub fn call(&self, logger: slog::Logger) -> Option<i32> {
        let resource_parser = KubeResourceParser::new(
            &self.resource_file,
            &self.resource_key,
        );

        let resource = match resource_parser.call() {
            Some(value) => {
                value
            },
            None => {
                return None
            }
        };

        // get list of all console and resource file names

        let kube_context = match resource.get("kube_context") {
            Some(value) => {
                info!(logger, "kube_context_ok"; "value" => value.as_str().unwrap());

                value.as_str().unwrap().to_owned()
            },
            None => {
                error!(logger, "kube_context_exception");

                return None
            }
        };

        let mut apply_errors = 0;
        let mut apply_success = 0;

        for file in self.files.iter() {
            match self._kubectl_apply(&file, &kube_context) {
                Ok(status) => {
                    if status.success() {
                        info!(logger, "kube_file_apply_ok"; "file" => file);

                        apply_success += 1;
                    } else {
                        error!(logger, "kube_file_apply_exception: {}", status);

                        apply_errors += 1;
                    }
                },
                Err(e) => {
                    error!(logger, "kube_file_apply_exception: {}", e);

                    apply_errors += 1;
                }
            }
        }

        // let apply_results = (apply_success, apply_errors);

        match (apply_success, apply_errors) {
            (_, 0) => {
                // all success
                return Some(0)
            }
            (0, _) => {
                // all errors
                return None
            },
            _ => {
                // some errors
                return None
            }
        };
    }

    fn _kubectl_apply(&self, kube_file: &String, kube_context: &String) -> Result<ExitStatus> {
        let kube_context_param = format!("--context={}", kube_context);

        let status = Command::new("kubectl")
            .args(&["apply", "-f", kube_file, &kube_context_param])
            .current_dir(FsRoot::call(&self.id))
            .status()?;

        Ok(status)
    }

}
