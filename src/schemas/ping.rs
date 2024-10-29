#[derive(Default, Debug)]
pub struct Ping {
    pub message: String,
}

#[juniper::graphql_object]
impl Ping {
    fn message(&self) -> &str {
        &self.message
    }
}
