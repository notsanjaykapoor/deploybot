use crate::schemas::root::Context;

#[derive(Default, Debug)]
pub struct Ping {
    pub message: String,
}

#[juniper::object(Context = Context)]
impl Ping {
    fn message(&self, context: &Context) -> &str {
        &self.message
    }
}
