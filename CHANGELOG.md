# Changelog

## [Unreleased][unreleased]

- Fixes a thread leakage on the nREPL host.  This was caused by `nr` by not
  closing the session as the last thing.

- Changes where the hosts files are searched from (**breaking**)

  The `nreplops-hosts.toml` files are now searched from the following
  directories in the given order:

  - the current directory
  - its parent directories
  - `${HOME}/Library/Application Support/nreplops`
  - `${XDG_CONFIG_HOME}/nreplops`
  - `${HOME}/.nreplops`

  Multiple hosts files are allowed.  In that case the files are merged together
  in reverse order so that the host file in the current directory (if any)
  dominates.

- Upgrades dependencies.  The current Minimum Supported Rust Version (MSRV)
  for this crate is 1.70.0.

[unreleased]: https://github.com/mjhanninen/nreplops-tool/compare/v0.0.10...main

## [Version 0.0.10][v0.0.10]

- Added a capability to define connection info in a host configuration file
  (`nreplops-hosts.toml`) and refer to them by a key

[v0.0.10]: https://github.com/mjhanninen/nreplops-tool/compare/v0.0.9...v0.0.10

## Version 0.0.9

- Added an experimental capability to tunnel through an SSH connection.  See the
  `--port` option.

