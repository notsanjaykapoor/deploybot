use super::fs::FsRoot;
use super::kube_resource::KubeResourceParser;

use std::fs;
use std::fs::File;
use std::io::Read;

#[derive(Debug)]
pub struct KubeFilesRewriter {
    id: String,
    image_tag_name: String,
    pub resource_file: String,
    pub resource_key: String,
}

impl KubeFilesRewriter {
    pub fn new(id: &String, resource_file: &String, resource_key: &String, image_tag_name: &String) -> KubeFilesRewriter {
        KubeFilesRewriter {
            id: id.to_owned(),
            resource_file: resource_file.to_owned(),
            resource_key: resource_key.to_owned(),
            image_tag_name: image_tag_name.to_owned(),
        }
    }

    pub fn call(&self) -> Option<Vec<String>> {
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

        let mut kube_files = Vec::new();

        match resource.get("console_files") {
            Some(value) => {
                for file_name in value.as_array().unwrap().to_vec().iter() {
                    kube_files.push(file_name.as_str().unwrap().to_owned());
                }
            },
            None => {}
        };

        match resource.get("resource_files") {
            Some(value) => {
                for file_name in value.as_array().unwrap().to_vec().iter() {
                    kube_files.push(file_name.as_str().unwrap().to_owned());
                }
            },
            None => {}
        };

        match self._files_update(kube_files) {
            Some(files) => {
                // println!("files_written: {:?}", files);

                return Some(files)
            },
            None => {
                return None
            }
        }
    }

    fn _files_update(&self, files: Vec<String>) -> Option<Vec<String>> {
        let mut files_copied = Vec::new();

        for file_name in files.iter() {
             match self._file_copy_replace(file_name.to_string()) {
                 Some(file) => {
                     files_copied.push(file.to_owned());
                 },
                 None => {
                     println!("file copy error: {}", file_name);

                     return None
                 }
             };
        }

        Some(files_copied)
    }

    fn _file_copy_replace(&self, file_name: String) -> Option<String> {
        let root_dir = FsRoot::call(&self.id);
        let file_name_current = format!("{}/{}", root_dir, file_name);
        let file_name_latest = format!("{}/{}.latest", root_dir, file_name);

        // ppen the path in read-only mode, returns `io::Result<File>`
        let mut file_current = match File::open(&file_name_current) {
            Err(_) => {
                print!("file not found: {}", &file_name_current);

                return None
            },
            Ok(file) => {
                file
            }
        };

        let mut input = String::new();

        match file_current.read_to_string(&mut input) {
            Err(_) => {
                return None
            },
            Ok(_) => {}
        };

        // replace :image_name with image that was just built
        let input_replaced = input.replace(":image_name", &self.image_tag_name.to_string());

        match fs::write(&file_name_latest, input_replaced.to_string()) {
            Err(_) => {
                return None
            },
            Ok(_) => {}
        };

        Some(file_name_latest)
    }

}
