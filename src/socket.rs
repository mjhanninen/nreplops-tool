use super::conn_expr;

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
                    // Already stopped
                } else {
                    // XXX(soija) Maybe use SIGTERM first?
                    let _ = p.kill();
                }
            }
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
