# Tunneling through SSH

The `nr` command has ability to tunnel the nREPL connection through an SSH
connection.  Forming the SSH connection is delegated to the `ssh` command
running in a child process.

## Old notes

- tunnel through ssh
  - [Thrussh](https://crates.io/crates/thrussh)
    - pure Rust
    - apache-2.0
    - not so good docs
  - [ssh2](https://crates.io/crates/ssh2) and [libssh2-sys](https://crates.io/crates/libssh2-sys)
    - wraps libssh2 C lib
    - mit or apache-2.0 (check static linking)
    - `Session::channel_direct_tcpip`
    - okay docs
  - [openssh](https://crates.io/crates/openssh)
    - wraps `ssh` client command (not lib)
    - mit or apache-2.0
    - oh! [jonhoo](https://github.com/jonhoo) goodness!
    - `~/.ssh/config` works
    - very command oriented
    - `Session::request_port_forward` (so a bit roundabout)
    - good docs
  - by using ControlMaster
  - calling ssh directly
    - use `-o BatchMode=yes`
      - disables interactive passwords and confirmations
      - does it disable ssh-agent kick as well?
    - use `-o ClearAllForwardings=yes`
      - ignores any other forwardings set out in ssh_config
    - port forwarding `-L`
      - `ssh -x -n -N -T -L <random-port>:<host>:<port>`
      - use openssh crate instead
    - stdin/out forwarding `-W`
      - `ssh -x -N -T -o ExitOnForwardFailure=yes -o ClearAllForwardings=yes -W <host>:<port>`
