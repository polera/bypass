use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum BypassError {
    #[error("Shortcut API error (HTTP {status}): {message}")]
    Api { status: u16, message: String },

    #[error("Name not found – no {resource_type} named '{name}' in this workspace")]
    NameNotFound { resource_type: String, name: String },

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Unsupported file format '{0}': use .yaml, .csv, or .xlsx")]
    UnsupportedFormat(String),
}
