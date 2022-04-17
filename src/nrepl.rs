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

use std::{
    io::{self, Read, Write},
    net::TcpStream,
};

use serde::{Deserialize, Serialize};

use crate::bencode;

#[derive(Debug)]
pub enum Op {
    Clone,
    Close,
    Eval,
}

impl std::str::FromStr for Op {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "clone" => Ok(Op::Clone),
            "close" => Ok(Op::Close),
            "eval" => Ok(Op::Eval),
            _ => Err("invalid operation"),
        }
    }
}

impl std::fmt::Display for Op {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            fmt,
            "{}",
            match *self {
                Op::Clone => "clone",
                Op::Close => "close",
                Op::Eval => "eval",
            }
        )
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct WireRequest {
    pub op: String,
    pub id: String,
    pub session: Option<String>,
    pub ns: Option<String>,
    pub code: Option<String>,
    pub line: Option<i32>,
    pub column: Option<i32>,
    pub file: Option<String>,
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
    stream: TcpStream,
    buffer: Vec<u8>,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            buffer: Default::default(),
        }
    }

    pub fn send(&mut self, request: &WireRequest) -> Result<(), io::Error> {
        let payload = serde_bencode::to_bytes(request).unwrap();
        self.stream.write_all(&payload)?;
        self.stream.flush()
    }

    pub fn try_recv(&mut self) -> Result<Response, RecvError> {
        let mut buffer = [0_u8; 4096];
        loop {
            match bencode::scan_next(&self.buffer) {
                Ok((_, len)) => {
                    let response = {
                        let input = &self.buffer[0..len];
                        let response: Response =
                            serde_bencode::from_bytes(input).unwrap();
                        response
                    };
                    self.buffer.copy_within(len.., 0);
                    self.buffer.truncate(self.buffer.len() - len);
                    return Ok(response);
                }
                Err(bencode::Error::UnexpectedEnd) => (),
                Err(bencode::Error::BadInput) => {
                    return Err(RecvError::BadInput);
                }
            }
            let bytes_read = self.stream.read(&mut buffer)?;
            if bytes_read == 0 {
                return Err(RecvError::HostDisconnected);
            }
            self.buffer.extend_from_slice(&buffer[0..bytes_read]);
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum RecvError {
    #[error("IO error")]
    Io(#[from] io::Error),
    #[error("bad input")]
    BadInput,
    #[error("unexpected disconnection by host")]
    HostDisconnected,
}
