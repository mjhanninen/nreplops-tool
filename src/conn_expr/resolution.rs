// conn_expr/resolution.rs
// Copyright 2022 Matti Hänninen
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not
// use this file except in compliance with the License. You may obtain a copy of
// the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations under
// the License.

use std::{fs, io, path, thread, time};

use crate::error::Error;

use super::conn_expr::ConnectionExpr;

#[derive(Debug)]
pub enum ConnectionExprSource {
    /// Use this connection expression.
    Direct(ConnectionExpr),
    /// Read the connection expression from the port file.
    PortFile {
        /// Use this instead of the nearest .nrepl-port file.
        path: Option<path::PathBuf>,
        /// If not available, give it this amount of time to appear.
        wait_for: Option<time::Duration>,
    },
}

impl From<ConnectionExpr> for ConnectionExprSource {
    fn from(e: ConnectionExpr) -> Self {
        ConnectionExprSource::Direct(e)
    }
}

impl From<&ConnectionExpr> for ConnectionExprSource {
    fn from(e: &ConnectionExpr) -> Self {
        e.clone().into()
    }
}

impl ConnectionExprSource {
    pub fn resolve_expr(&self) -> Result<ConnectionExpr, Error> {
        const THROTTLING_DELAY: time::Duration =
            time::Duration::from_millis(50);
        match self {
            ConnectionExprSource::Direct(e) => Ok(e.clone()),
            ConnectionExprSource::PortFile {
                path,
                wait_for: None,
            } => try_load_from_port_file(path.as_ref()),
            ConnectionExprSource::PortFile {
                path,
                wait_for: Some(duration),
            } => {
                let deadline = time::SystemTime::now() + *duration;
                loop {
                    match try_load_from_port_file(path.as_ref()) {
                        Ok(r) => return Ok(r),
                        Err(e) => match e {
                            Error::NotSpecified | Error::NotFound(_) => {
                                if time::SystemTime::now() >= deadline {
                                    return Err(Error::PortFileTimeout);
                                }
                            }
                            _ => return Err(e),
                        },
                    }
                    thread::sleep(THROTTLING_DELAY);
                }
            }
        }
    }
}

fn try_load_from_port_file(
    given: Option<impl AsRef<path::Path>>,
) -> Result<ConnectionExpr, Error> {
    if let Some(f) = given {
        load_from_port_file(f)
    } else if let Some(ref f) = find_port_file().map_err(|_| Error::Unknown)? {
        load_from_port_file(f)
    } else {
        Err(Error::NotSpecified)
    }
}

fn find_port_file() -> io::Result<Option<path::PathBuf>> {
    let current_dir = path::PathBuf::from(".").canonicalize()?;
    for dir in current_dir.ancestors() {
        let mut path = path::PathBuf::from(dir);
        path.push(".nrepl-port");
        if path.is_file() {
            return Ok(Some(path));
        }
    }
    Ok(None)
}

fn load_from_port_file(
    path: impl AsRef<path::Path>,
) -> Result<ConnectionExpr, Error> {
    let path = path.as_ref();
    fs::read_to_string(path)
        .map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                Error::NotFound(path.to_string_lossy().into())
            } else {
                Error::CannotReadFile(path.to_string_lossy().into())
            }
        })?
        .trim()
        .parse()
        .map_err(|_| Error::CannotParsePortFile(path.to_string_lossy().into()))
}
