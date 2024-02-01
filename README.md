# nreplops-tool (nr)

nreplops-tool (`nr`) is a non-interactive nREPL client, optimized for use in
shell scripts and command-line environments.

## Project status

**Inactively developed:** This is project is under development (i.e. not dead,
not feature complete yet, big changes may happen) but receives only limited
attention.  The tool itself is used daily in production environments by the
author.

Please see [PLAN.md](./PLAN.md) for project goals and planned features.

## Installation

### Homebrew

```sh
brew install mjhanninen/sour/nreplops-tool
```

### Cargo

The Minimum Supported Rust Version (MSRV) is 1.70.0.

```sh
cargo install nreplops-tool
```

### Building from sources

The Minimum Supported Rust Version (MSRV) is 1.70.0.

Clone the repository:

```sh
git clone https://github.com/mjhanninen/nreplops-tool.git
```

Use `cargo` to build and install the tool:

```sh
cargo install --path nr
```

## License

Copyright 2022 Matti Hänninen

Licensed under the Apache License 2.0

Please see the [LICENSE](./LICENSE) and [NOTICE](./NOTICE) files.
