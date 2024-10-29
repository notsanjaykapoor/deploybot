use serde::Serialize;

#[derive(Default, Debug, Serialize)]
pub struct DeployResult {
    pub code: i32,
    pub id: String,
}

#[juniper::graphql_object]
impl DeployResult {
    fn code(&self) -> i32 {
        self.code
    }

    fn id(&self) -> &str {
        &self.id
    }
}
