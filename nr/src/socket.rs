// socket.rs
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

use super::{
  error::Error,
  routes::{Route, Routes},
};

use std::{
  io::{self, Read, Write},
  net::{self, TcpStream},
  process::{Child, Command, Stdio},
};

#[derive(Debug)]
pub enum Socket {
  TcpStream(TcpStream),
  SshClient(Child),
}

impl From<TcpStream> for Socket {
  fn from(s: TcpStream) -> Self {
    Socket::TcpStream(s)
  }
}

impl Socket {
  pub fn borrow_mut_read(&mut self) -> &mut dyn Read {
    match *self {
      Socket::TcpStream(ref mut s) => s,
      Socket::SshClient(ref mut p) => {
        p.stdout.as_mut().expect("child process's stdout is piped")
      }
    }
  }

  pub fn borrow_mut_write(&mut self) -> &mut dyn Write {
    match *self {
      Socket::TcpStream(ref mut s) => s,
      Socket::SshClient(ref mut p) => {
        p.stdin.as_mut().expect("child process's stdin is piped")
      }
    }
  }
}

impl Drop for Socket {
  fn drop(&mut self) {
    match *self {
      Socket::TcpStream(ref mut s) => {
        let _ignore = s.shutdown(net::Shutdown::Both);
      }
      Socket::SshClient(ref mut p) => {
        if let Ok(Some(_)) = p.try_wait() {
          // The child process has already stopped
        } else {
          // XXX(soija) Maybe use SIGTERM first?
          let _ = p.kill();
        }
      }
    }
  }
}

pub fn connect(mut routes: Routes) -> Result<Socket, Error> {
  let first_route = routes.next().expect("there is at least one route");
  match connect_impl(&first_route) {
    Ok(socket) => Ok(socket),
    Err(first_err) if first_err.kind() == io::ErrorKind::ConnectionRefused => {
      for route in routes {
        match connect_impl(&route) {
          Ok(socket) => {
            return Ok(socket);
          }
          Err(e) if e.kind() == io::ErrorKind::ConnectionRefused => {
            continue;
          }
          Err(err) => return Err(Error::FailedToConnectToHost(err)),
        }
      }
      Err(Error::FailedToConnectToHost(first_err))
    }
    Err(err) => Err(Error::FailedToConnectToHost(err)),
  }
}

fn connect_impl(route: &Route) -> Result<Socket, io::Error> {
  use Route::*;
  match *route {
    Direct(ip) => {
      let s = TcpStream::connect(ip)?;
      s.set_nodelay(true)?;
      Ok(Socket::from(s))
    }
    Tunneled(ref opts) => {
      let mut cmd = Command::new("ssh");
      cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .arg("-x")
        .arg("-N")
        .arg("-T")
        .arg("-o")
        .arg("ExitOnForwardFailure=yes")
        .arg("-o")
        .arg("ClearAllForwardings=yes")
        .arg("-o")
        .arg("ConnectTimeout=5")
        .arg("-W")
        .arg(format!("{}:{}", opts.host_addr, opts.host_port));
      if let Some(ref user) = opts.ssh_user {
        cmd.arg("-l").arg(user);
      }
      if let Some(ref port) = opts.ssh_port {
        cmd.arg("-p").arg(port.to_string());
      }
      cmd.arg(opts.ssh_addr.to_string());
      //
      // XXX(soija) Here we are content with just being able to spawn the
      //            child process successfully and don't verify that a
      //            forwarded connection is actually formed.
      //
      //            This is okay for now as a failing ssh client seems to
      //            cause a decent enough error when we try to read from
      //            or write to the socket on this side.  However, this
      //            needs to be solved somehow before it is possible to
      //            knock multiple ports (in case I want to retain that
      //            feature).
      //
      Ok(Socket::SshClient(cmd.spawn()?))
    }
  }
}
