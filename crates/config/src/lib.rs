mod error;
pub mod model;
pub use error::*;
use serde::Deserialize;

use std::path::Path;

use crate::model::Config;

impl<Source> Config<Source>
where
    Source: for<'de> Deserialize<'de>,
{
    pub fn load(
        path: impl AsRef<Path>,
        max_depth: usize,
        follow_symlinks: bool,
        buffer: &mut Vec<Config<Source>>,
    ) -> Result<(), Error> {
        fn load<Source>(
            path: impl AsRef<Path>,
            max_depth: usize,
            current_depth: usize,
            follow_symlinks: bool,
            buffer: &mut Vec<Config<Source>>,
        ) -> Result<(), Error>
        where
            Source: for<'de> Deserialize<'de>,
        {
            let path = path.as_ref();
            if path.try_exists()? {
                if path.is_dir() {
                    if current_depth + 1 >= max_depth {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "maximum depth reached",
                        )
                        .into());
                    }
                    let current_depth = current_depth + 1;
                    for f in path.read_dir()? {
                        let path = f?.path();
                        if !path.is_symlink() || follow_symlinks {
                            load(path, max_depth, current_depth, follow_symlinks, buffer)?;
                        } else {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                "symlinks not allowed",
                            )
                            .into());
                        }
                    }
                } else {
                    if !path.is_symlink() || follow_symlinks {
                        let data = std::fs::read_to_string(path)?;
                        let mut config: Config<Source> = toml::from_str(&data)?;
                        config.source = Some(path.to_owned());
                        buffer.push(config);
                    } else {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "symlinks not allowed",
                        )
                        .into());
                    }
                }

                todo!()
            } else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "missing config file or directory",
                )
                .into());
            }
        }
        load(path, max_depth, 0, follow_symlinks, buffer)
    }
}
