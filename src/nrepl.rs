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

use std::io::ErrorKind;

use serde::{Deserialize, Serialize};

use super::socket::Socket;
use crate::{bencode, error::Error};

#[derive(Debug)]
pub struct Session {
    // The session can own the connection as We do not support multiple sessions
    // even though the nREPL protocol would permit it.
    connection: Connection,
    session_id: Box<str>,
    request_count: usize,
}

impl Session {
    pub fn close(mut self) -> Result<Connection, Error> {
        let id = format!("{}:close", self.session_id);
        self.connection.send(WireRequest {
            op: Op::Close,
            id: &id,
            session: Some(&self.session_id),
            ns: None,
            code: None,
            line: None,
            column: None,
            file: None,
        })?;
        #[allow(clippy::blocks_in_if_conditions)]
        while self
            .connection
            .recv(|r| Ok(!(r.matches(&id) && r.has_status("session-closed"))))?
        {
        }
        Ok(self.connection)
    }

    pub fn eval<F>(
        &mut self,
        code: &str,
        file_name: Option<&str>,
        line: Option<usize>,
        column: Option<usize>,
        mut handler: F,
    ) -> Result<(), Error>
    where
        F: FnMut(Response) -> Result<(), Error>,
    {
        self.request_count += 1;
        let id = format!("{}:{}", self.session_id, self.request_count);
        self.connection.send(WireRequest {
            id: &id,
            op: Op::Eval,
            session: Some(&self.session_id),
            ns: None,
            code: Some(code),
            line: line.map(|n| n.try_into().unwrap_or_default()),
            column: column.map(|n| n.try_into().unwrap_or_default()),
            file: file_name,
        })?;
        #[allow(clippy::blocks_in_if_conditions)]
        while self.connection.recv(|r| {
            if !r.matches(&id) {
                return Ok(true);
            }
            handler(Response {
                value: r.value.as_deref(),
                out: r.out.as_deref(),
                err: r.err.as_deref(),
                ex: r.ex.as_deref(),
                root_ex: r.root_ex.as_deref(),
            })?;
            Ok(!r.has_status("done"))
        })? {}
        Ok(())
    }
}

#[derive(Debug)]
pub struct Response<'a> {
    pub value: Option<&'a str>,
    pub ex: Option<&'a str>,
    pub root_ex: Option<&'a str>,
    pub out: Option<&'a str>,
    pub err: Option<&'a str>,
}

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

fn serialize_op<S: serde::Serializer>(
    op: &Op,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(op.as_str())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct WireRequest<'a> {
    #[serde(serialize_with = "serialize_op")]
    pub op: Op,
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
pub struct WireResponse {
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

impl WireResponse {
    pub fn matches(&self, id: &str) -> bool {
        self.id.as_ref().map(|our| our == id).unwrap_or(false)
    }

    pub fn has_status(&self, label: &str) -> bool {
        self.status
            .as_ref()
            .map(|labels| labels.iter().any(|our| our == label))
            .unwrap_or(false)
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

    pub fn session(mut self) -> Result<Session, Error> {
        self.send(WireRequest {
            op: Op::Clone,
            id: "",
            session: None,
            ns: None,
            code: None,
            line: None,
            column: None,
            file: None,
        })?;
        let session_id = self.recv(|response| {
            if let Some(session) = response.new_session.as_deref() {
                Ok(session.to_owned().into_boxed_str())
            } else {
                Err(Error::UnexptectedResponse)
            }
        })?;
        Ok(Session {
            connection: self,
            session_id,
            request_count: 0,
        })
    }

    fn send(&mut self, request: WireRequest) -> Result<(), Error> {
        let payload = serde_bencode::to_bytes(&request).unwrap();
        let w = self.socket.borrow_mut_write();
        w.write_all(&payload).map_err(Error::CannotSendToHost)?;
        w.flush().map_err(Error::CannotSendToHost)
    }

    fn recv<F, V>(&mut self, mut handler: F) -> Result<V, Error>
    where
        F: FnMut(&WireResponse) -> Result<V, Error>,
    {
        let mut buffer = [0_u8; 4096];
        loop {
            match bencode::scan_next(&self.buffer) {
                Ok((_, len)) => {
                    let result =
                        serde_bencode::from_bytes(&self.buffer[0..len])
                            .map_err(|_| Error::CorruptedResponse);
                    self.buffer.copy_within(len.., 0);
                    self.buffer.truncate(self.buffer.len() - len);
                    return handler(&result?);
                }
                Err(bencode::Error::BadInput) => {
                    return Err(Error::CorruptedResponse);
                }
                Err(bencode::Error::UnexpectedEnd) => {}
            }
            match self.socket.borrow_mut_read().read(&mut buffer) {
                Ok(0) => return Err(Error::HostDisconnected),
                Ok(len) => self.buffer.extend_from_slice(&buffer[0..len]),
                Err(e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => return Err(Error::CannotReceiveFromHost(e)),
            }
        }
    }
}
