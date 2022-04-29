use super::conn_expr;

use std::{
    io::{self, Read, Write},
    net::TcpStream,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

#[derive(Debug)]
pub enum Socket {
    TcpStream(TcpStream),
    SshClient {
        process: Child,
        stdin: ChildStdin,
        stdout: ChildStdout,
    },
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
            Socket::SshClient {
                stdout: ref mut r, ..
            } => r,
        }
    }

    pub fn borrow_mut_write(&mut self) -> &mut dyn Write {
        match *self {
            Socket::TcpStream(ref mut s) => s,
            Socket::SshClient {
                stdin: ref mut w, ..
            } => w,
        }
    }
}

pub fn connect(routes: conn_expr::Routes) -> Result<Socket, io::Error> {
    let mut last_err = None;
    for route in routes {
        match connect_impl(&route) {
            Ok(s) => {
                return Ok(s);
            }
            Err(e) if e.kind() == io::ErrorKind::ConnectionRefused => {
                last_err = Some(e);
                continue;
            }
            Err(e) => return Err(e),
        }
    }
    Err(last_err.expect("at least one failed connection attempt"))
}

fn connect_impl(route: &conn_expr::Route) -> Result<Socket, io::Error> {
    use conn_expr::Route::*;
    match *route {
        Direct(ip) => {
            let s = TcpStream::connect(ip)?;
            s.set_nodelay(true)?;
            Ok(Socket::from(s))
        }
        // XXX(soija) Cool, this actually works!!! However, this leaves zombie
        //            ssh clients behind.  Need to figure out how to kill them.
        //            (Add Drop to Socket).
        Tunneled(ref opts) => {
            let mut cmd = Command::new("ssh");
            cmd.stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .arg("-x")
                .arg("-N")
                .arg("-T")
                .arg("-o")
                .arg("ExitOnForwardFailure=yes")
                .arg("-o")
                .arg("ClearAllForwardings=yes")
                .arg("-W")
                .arg(format!("{}:{}", opts.host_addr, opts.host_port));
            if let Some(ref user) = opts.ssh_user {
                cmd.arg("-l").arg(user);
            }
            if let Some(ref port) = opts.ssh_port {
                cmd.arg("-p").arg(port.to_string());
            }
            cmd.arg(opts.ssh_addr.to_string());
            let mut process = cmd.spawn()?;
            let stdin =
                process.stdin.take().expect("child process has piped stdin");
            let stdout = process
                .stdout
                .take()
                .expect("child process has piped stdout");
            Ok(Socket::SshClient {
                process,
                stdin,
                stdout,
            })
        }
    }
}
