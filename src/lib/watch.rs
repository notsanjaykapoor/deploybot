use super::kube_resource::{KubeResourceParser, KubeResourceResolve};

use std::io::{Error, ErrorKind, Result};
use std::process::{Command};
use std::str;

#[derive(Debug)]
pub struct WatchObject {
    pub cmd: String,
    pub sleep: u64,
    pub wait: u64,
}

#[derive(Debug)]
pub struct WatchStage {
    pub id: String,
    pub resource_file: String,
    pub resource_key: String,
    pub logger: slog::Logger,
}

impl WatchObject {

    pub fn call(&self) -> Result<i32> {
        let mut cmd_list: Vec<_> = self.cmd.split(" ").collect();
        let cmd_name = cmd_list.remove(0);

        let output = Command::new(cmd_name)
            .args(cmd_list)
            .output()?;

        let output_str = str::from_utf8(&output.stdout).unwrap();

        // println!("output status: {}", output.status);
        // println!("output stdout: {}", output_str);

        if output_str.contains("successfully rolled out") {
            return Ok(0)
        }

        Ok(1)
    }

}

impl WatchStage {

    pub fn new(id: &String, resource_path: &String, logger: slog::Logger) -> WatchStage {
        let resource_vec: Vec<_> = resource_path.split(":").collect();
        let resource_file = KubeResourceResolve::call(id, resource_path);

        WatchStage {
            id: id.to_owned(),
            resource_file: resource_file.to_owned(),
            resource_key: resource_vec[1].to_owned(),
            logger: logger,
        }
    }

    pub fn call(&self) -> Result<Vec<WatchObject>> {
        let resource_parser = KubeResourceParser::new(
            &self.resource_file,
            &self.resource_key,
        );

        // println!("watch {}, {}", self.resource_file, self.resource_key);

        let resource = match resource_parser.call() {
            Some(value) => {
                value
            },
            None => {
                return Err(Error::new(ErrorKind::Other, "watches missing"))
            }
        };

        // build list of watch objects

        match resource.get("watches") {
            Some(value) => {
                let v: Vec<WatchObject> = value.as_array().unwrap().to_vec().iter()
                    .map(|o| self._watch_object(o) )
                    .filter_map(|o| o.ok())
                    .collect();

                return Ok(v)
            },
            None => {
                return Err(Error::new(ErrorKind::Other, "watches missing"))
            }
        };
    }

    fn _watch_object(&self, watch: &toml::Value) -> Result<WatchObject> {
        let cmd = match watch.get("cmd") {
            None => {
                return Err(Error::new(ErrorKind::Other, "watch cmd required"))
            },
            Some(value) => {
                value.as_str().unwrap().to_string() // remove string quotes
            }
        };

        let wait = match watch.get("wait") {
            None => {
                60
            },
            Some(value) => {
                value.as_integer().unwrap()
            }
        };

        let sleep = match watch.get("sleep") {
            None => {
                20
            },
            Some(value) => {
                value.as_integer().unwrap()
            }
        };

        Ok(WatchObject {
            cmd: cmd,
            sleep: sleep as u64,
            wait: wait as u64,
        })
    }

}
