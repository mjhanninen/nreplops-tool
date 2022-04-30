# Port expression

## Old notes

- port file syntax
  - `<port>`
    - a local nREPL host
  - `<remote-host>:<remote-port>`
    - a remote nREPL host
  - `<ssh-host>:<remote-host>:<remote-port>`
    - a remote nREPL host, reachable via SSH host (port 22 or from `.ssh/config`)
  - `<ssh-host>:<ssh-port>:<remote-host>:<remove-port>`
    - a remote nREPL host, reachable via SSH host on specific port
  - `<ssh-host>:<ssh-port-list>:<remote-host>:<remove-port>`
    - a remote nREPL host, reachable via SSH host on a port within a given range
    - e.g. `vpn.biscuit.example.com:1024-1027,1032:[::1]:7000`
    - obvious but name resolution post-poned to the remote side
  - `<ssh-user>@<ssh-host>:<ssh-port>:<remote-host>:<remote-port>`
