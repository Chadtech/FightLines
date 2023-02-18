pub struct Error {
    pub title: String,
    pub msg: String,
}

impl Error {
    pub fn throw<T>(title: String, msg: String) -> Result<T, Error> {
        Err(Error::new(title, msg))
    }

    pub fn new(title: String, msg: String) -> Error {
        Error { title, msg }
    }
}
