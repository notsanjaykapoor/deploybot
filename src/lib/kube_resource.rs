use super::fs::FsRoot;

#[derive(Debug)]
pub struct KubeResourceParser {
    pub resource_file: String,
    pub resource_key: String,
}

pub struct KubeResourceResolve {}

impl KubeResourceParser {
    pub fn new(resource_file: &String, resource_key: &String) -> KubeResourceParser {
        KubeResourceParser {
            resource_file: resource_file.to_owned(),
            resource_key: resource_key.to_owned(),
        }
    }

    pub fn call(&self) -> Option<toml::Value> {
        let toml_string = match std::fs::read_to_string(self.resource_file.to_string()) {
            Err(_) => {
                return None
            },
            Ok(value) => {
                value
            }

        };

        // println!("toml_string: {:?}", toml_string);

        match self._toml_parse(toml_string) {
            Some(value) => {
                // println!("value: {:?}", value);

                return Some(value)
            },
            None => {
                return None
            }
        };
    }

    fn _toml_parse(&self, toml_string: String) -> Option<toml::Value> {
        let toml_object: toml::Value = match toml::from_str(&toml_string) {
            Err(_) => {
                return None
            },
            Ok(value) => {
                value
            }
        };

        // println!("toml_object: {:?}", toml_object);

        let resources_list = match toml_object.get("resources") {
            None => {
                return None
            },
            Some(value) => {
                value.as_array().unwrap()
            }
        };

        // iterate resources_list to find 'resource_key' match

        let resource = resources_list.iter().find(
            |&resource| resource.get("name").unwrap().as_str().unwrap() == self.resource_key
        );

        // println!("resource: {:?}", resource);

        match resource {
            Some(value) => {
                return Some(value.clone())
            },
            None => {
                return None
            }
        }
    }

}

impl KubeResourceResolve {
    pub fn call(id: &str, resource_path: &str) -> String {
        // e.g. "kubernetes/resources.toml:api-staging"
        let resource_vec: Vec<_> = resource_path.split(":").collect();

        // resolve resource_file using fs root
        let root_dir = FsRoot::call(id);

        format!("{}/{}", root_dir, resource_vec[0])
    }
}
