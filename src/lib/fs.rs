use std::fs;
use std::fs::File;

const DEPLOYBOT_TMP_DIR: &str = "/var/tmp/deploybot";

#[derive(Debug)]
pub struct FsRemove {}

#[derive(Debug)]
pub struct FsRoot {}

#[derive(Debug)]
pub struct FsTouch {}

impl FsRemove {
    pub fn call(id: &str) -> Option<u32> {
        let path = format!("{}.txt", FsRoot::call(id));

        match fs::remove_file(path) {
            _ => {}
        };

        Some(0)
    }
}

impl FsRoot {
    pub fn call(id: &str) -> String {
        format!("{}/{}", DEPLOYBOT_TMP_DIR, id)
    }
}

impl FsTouch {
    pub fn call(id: &str) -> Option<u32> {
        let path = format!("{}.txt", FsRoot::call(id));

        match File::create(path) {
            _ => {}
        }

        Some(0)
    }
}
