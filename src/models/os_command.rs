#[derive(Clone)]
pub struct OsCommand {
    pub expect: String,
    pub send: String
}

impl OsCommand {
    pub fn new<E: AsRef<str>, S: AsRef<str>>(expect: E, send: S) -> OsCommand {
        OsCommand {
            expect: expect.as_ref().to_string(),
            send: send.as_ref().to_string(),
        }
    }
}