use std::error::Error;
use std::fmt;

pub enum SettingsError {
    FileParse {
        path: Option<String>,
        cause: Box<dyn Error + Send + Sync>,
    },
    Message(String),
    MissingValue(String),
}

impl fmt::Display for SettingsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SettingsError::FileParse { path, cause } => {
                write!(f, "FileParse {}", cause)?;

                if let Some(path) = path {
                    write!(f, " in {}", path)?;
                }

                Ok(())
            }
            SettingsError::Message(msg) => write!(f, "{}", msg),
            SettingsError::MissingValue(path) => write!(f, "Missing settings value at {}", path),
        }
    }
}

impl std::fmt::Debug for SettingsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}
