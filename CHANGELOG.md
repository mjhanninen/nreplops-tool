# Changelog

## [Unreleased][unreleased]

[unreleased]: https://github.com/mjhanninen/nreplops-tool/compare/v0.2.0...main

## [Version 0.2.0][v0.2.0]

- **Breaking**: The shebang mode flag `-!` now requires the source file to be
  specified as the first positional argument. Piping from stdin is no longer
  supported.

- Enhancement: The shebang mode flag `-!` now supports an optional version
  assertion. This can be either a minimum point version in a non-breaking version
  range (e.g., `-! 0.2` for versions >= 0.2.0 and < 0.3.0) or a specific version
  range (e.g., `-! 1.2.3..4.5.6` for versions >= 1.2.3 and < 4.5.6).

[v0.2.0]: https://github.com/mjhanninen/nreplops-tool/compare/v0.1.2...v0.2.0}

## [Version 0.1.2][v0.1.2]

- Fixes spurious double error caused by trying to close the session when the
  connection has already failed.

[v0.1.2]: https://github.com/mjhanninen/nreplops-tool/compare/v0.1.1...v0.1.2}

## [Version 0.1.1][v0.1.1]

- Fixes the thread leakage issue on the nREPL host further: attempts to close
  the session in failure cases too.

- Upgrades dependecies. MSRV is 1.70.0.

[v0.1.1]: https://github.com/mjhanninen/nreplops-tool/compare/v0.1.0...v0.1.1}

## [Version 0.1.0][v0.1.0]

- Fixes a thread leakage on the nREPL host.  This was caused by `nr` not
  closing the nREPL session at exit.

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

[v0.1.0]: https://github.com/mjhanninen/nreplops-tool/compare/v0.0.10...v0.1.0}

## [Version 0.0.10][v0.0.10]

- Added a capability to define connection info in a host configuration file
  (`nreplops-hosts.toml`) and refer to them by a key

[v0.0.10]: https://github.com/mjhanninen/nreplops-tool/compare/v0.0.9...v0.0.10

## Version 0.0.9

- Added an experimental capability to tunnel through an SSH connection.  See the
  `--port` option.

