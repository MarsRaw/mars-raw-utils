use anyhow;

#[derive(anyhow::Error, Debug, Clone, PartialEq, Eq)]
pub enum Error {
    RemoteError(String),
    NoNetwork(String),
    FileFoundLocal(String),
    MissionNotSupported(String),
    NotDownloading(String),
    UnknownError(String),
    ParseFailure(String),
}

#[macro_export]
macro_rules! why {
    ($($arg:tt)*) => {
        format!($($arg)*)
    };
}

#[macro_export]
macro_rules! remote_error {
    ($why:expr) => {{
        FetchError::RemoteError(why!($why))
    }};
}

#[macro_export]
macro_rules! no_network {
    ($why:expr) => {
        FetchError::NoNetwork(why!($why))
    };
}

#[macro_export]
macro_rules! file_found_local {
    ($file_path:expr) => {
        FetchError::FileFoundLocal(why!(
            "File exists on destination filesystem: {}",
            $file_path
        ))
    };
}

#[macro_export]
macro_rules! mission_not_supported {
    ($mission:expr) => {{
        FetchError::MissionNotSupported(why!("Mission not supported: {}", $mission))
    }};
}

#[macro_export]
macro_rules! not_downloading {
    () => {{
        FetchError::NotDownloading(why!("Skipping download; User requested to list only"))
    }};
}

#[macro_export]
macro_rules! unknown_error {
    ($why:expr) => {{
        FetchError::UnknownError(why!($why))
    }};
}

#[macro_export]
macro_rules! parse_failure {
    ($($arg:tt)*) => {
        FetchError::ParseFailure(why!($($arg)*))
    };
}
