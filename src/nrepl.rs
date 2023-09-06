// nrepl.rs
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

use std::io::{self, ErrorKind};

use serde::{Deserialize, Serialize};

use super::socket::Socket;
use crate::bencode;

#[derive(Debug)]
pub enum Op {
    Clone,
    Close,
    Eval,
}

impl Op {
    pub fn as_str(&self) -> &'static str {
        match *self {
            Op::Clone => "clone",
            Op::Close => "close",
            Op::Eval => "eval",
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct WireRequest<'a> {
    pub op: &'a str,
    pub id: &'a str,
    pub session: Option<&'a str>,
    pub ns: Option<&'a str>,
    pub code: Option<&'a str>,
    pub line: Option<i32>,
    pub column: Option<i32>,
    pub file: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Response {
    pub session: String,
    pub id: Option<String>,
    pub status: Option<Vec<String>>,
    pub new_session: Option<String>,
    pub value: Option<String>,
    pub ex: Option<String>,
    pub root_ex: Option<String>,
    pub out: Option<String>,
    pub err: Option<String>,
}

impl Response {
    pub fn has_status(&self, label: &str) -> bool {
        if let Some(ref ss) = self.status {
            for s in ss.iter() {
                if s == label {
                    return true;
                }
            }
        }
        false
    }
}

#[derive(Debug)]
pub struct Connection {
    socket: Socket,
    buffer: Vec<u8>,
}

impl Connection {
    pub fn new(socket: Socket) -> Self {
        Self {
            socket,
            buffer: Default::default(),
        }
    }

    pub fn send(&mut self, request: &WireRequest) -> Result<(), io::Error> {
        let payload = serde_bencode::to_bytes(request).unwrap();
        let w = self.socket.borrow_mut_write();
        w.write_all(&payload)?;
        w.flush()
    }

    pub fn try_recv(&mut self) -> Result<Response, RecvError> {
        let mut buffer = [0_u8; 4096];
        loop {
            match bencode::scan_next(&self.buffer) {
                Ok((_, len)) => {
                    let response =
                        serde_bencode::from_bytes(&self.buffer[0..len])
                            .map_err(|_| RecvError::CorruptedResponse);
                    self.buffer.copy_within(len.., 0);
                    self.buffer.truncate(self.buffer.len() - len);
                    return response;
                }
                Err(bencode::Error::BadInput) => {
                    return Err(RecvError::CorruptedResponse);
                }
                Err(bencode::Error::UnexpectedEnd) => {}
            }
            match self.socket.borrow_mut_read().read(&mut buffer) {
                Ok(0) => return Err(RecvError::HostDisconnected),
                Ok(len) => self.buffer.extend_from_slice(&buffer[0..len]),
                Err(e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => return Err(RecvError::Io(e)),
            }
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum RecvError {
    #[error("IO error")]
    Io(#[from] io::Error),
    #[error("corrupted response")]
    CorruptedResponse,
    #[error("unexpected disconnection by host")]
    HostDisconnected,
}
