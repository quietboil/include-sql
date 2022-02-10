pub(crate) type Result<T> = std::result::Result<T, Error>;

/// A list of possible include-sql errors
#[derive(Debug)]
pub(crate) enum Error {
    Sql(String),
    IO(std::io::Error)
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &*self {
            Error::Sql(msg) => write!(f, "{}", msg),
            Error::IO(err) => err.fmt(f)
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Error::Sql(_) => None,
            Error::IO(ref err) => Some(err)
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::IO(err)
    }
}

/// A helper that creates an error with a description
pub(crate) fn new(msg: String) -> Error {
    Error::Sql(msg)
}
