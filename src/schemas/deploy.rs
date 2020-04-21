use crate::schemas::root::Context;

use serde::Serialize;

#[derive(Default, Debug, Serialize)]
pub struct DeployResult {
    pub code: i32,
    pub id: String,
}

#[juniper::object(Context = Context)]
impl DeployResult {
    fn code(&self, context: &Context) -> i32 {
        self.code
    }

    fn id(&self, context: &Context) -> &str {
        &self.id
    }
}
