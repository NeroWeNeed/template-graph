use template_engine_commons::format;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Format(#[from] format::FormatError),
    #[error(transparent)]
    Notify(#[from] notify::Error),
}
