# Changelog

## Unreleased

- Changes to hosts file search: The `nreplops-hosts.toml` files are now searched
  for from the following directories:

  - the current directory
  - its parents
  - `${HOME}/Library/Application Support/nreplops` (on macOS)
  - `${XDG_CONFIG_HOME}/nreplops`
  - `${HOME}/.nreplops`

  Multiple files are allowed.  In that case the files are merged together in
  reverse order.

- Updgraded dependencies

## Version 0.0.10

- Added a capability to define connection info in a host configuration file
  (`nreplops-hosts.toml`) and refer to them by a key

## Version 0.0.9

- Added an experimental capability to tunnel through an SSH connection.  See the
  `--port` option.
