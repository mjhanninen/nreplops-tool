# nreplops-tool (nr)

nreplops-tool (`nr`) is a non-interactive nREPL client designed to be used in
shell scripts and on the command-line.

## Project status

**Inactively developed:** This is project is under development (i.e. not dead,
not feature complete yet, big changes may happen) but receives only limited
attention.  The tool itself is used daily in production environments by the
author.

Please see [PLAN.md](./PLAN.md) for project goals and planned features.

## Try it out in 1 minute

This example assumes you are able to install packages with Homebrew.  See the
[Installation](#installation) section below for other options.

Start by installing nreplops-tool (`nr`) and Babashka (`bb`):

```sh
brew install mjhanninen/sour/nreplops-tool borkdude/brew/babashka
```

Launch a Babashka nREPL server (that listens on the port 1667 by default):

```sh
bb nrepl-server
```

Open another terminal and evaluate an expression with `nr`:

```sh
nr -p 1667 -e '(println "Hello, world!")'
```

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
cargo install --path nreplops-tool
```

## License

Copyright 2022 Matti Hänninen

Licensed under the Apache License 2.0

Please see the [LICENSE](./LICENSE) and [NOTICE](./NOTICE) files.
