# Planned features and goals

## Project goals

- Easy of use in shell scripts
- Consistency and interoperability with selected CLI tools:
  - [Babashka][babashka]
  - [Jet][jet]
  - [jq][jq]

[babashka]: https://github.com/babashka/babashka
[jet]: https://github.com/borkdude/jet
[jq]: https://github.com/stedolan/jq

## More important features

- `^C` interrupts evaluation (now: runaway)
- Implement timeout; we already have `--timeout` option without implementation, related to `^C`
- Proper argument handling and interpolation (see below)
- Reuse tunneled connection (see OpenSSH ControlMaster)
- Hostname assertions for SSH connections
- Check that SSH client is present and one that we support
- Short-circuiting: given `-e A -e B` when `A` fails don't evaluate `B`; opt-in
- `--exprs-to <sink>` for echoing sent expressions to `<sink>`
- `--stdin-to <sink>` for echoing sent input to `<sink>`
- `--log-to <sink>` write an execution log to a file
- `--log` write an execution log to a file named by the source file
- ~~`-!` takes minimum version~~ (done)
- `--production` with optional "are you sure?" mechanism
- `--dry-run` for debugging (combine with `--exprs-to` to see what would be sent)
- Return a distinct error code if evaluation throws; opt-in (now: 0)
- Windows support

## Less important features

- Capture arguments from environment (now: only from command line)
- Host name assertion (safer, esp. with port forwarding vpns)
- Colored output: on/off/auto, defaults to auto, match with jq colors
- Pretty-printing: on/off/auto, defaults to auto
- JSON encoded output
- YAML encoded output
- table output
- `--help` renders script's help when invoked via shebang
- Tooling for rendering script "docstrings" into Markdown
- Cache successfully resolved route to reduce unnecesary knocking
- [socket prepl support](./notes/prepl.md)
- Merge `--port` and `--port-file` options
  - `--port` already handles host aliases, so why not port file paths
- Rewrite rules (e.g `clojure.pprint/print` → `puget.cprint`)
  - conditional on stdout or stderr being connected to tty locally
- `--watch` the script and input files and resubmit upon change
- **Skip SSH fingerprint**: When tunneling through a stable localhost port (can
  happen with VPN setups, for example) to multiple different remote hosts you
  easily end up in a sitation in which the host fingerprint check fails.  Add an
  option to host configuration (`nreplops-hosts.toml`) to allow skipping this
  check.  (Effectively `-o StrictHostKeyChecking=no`.)  Alternatively document
  how to achieve the same by changing `~/.ssh/config`.
- **Run against multiple servers**: Enable running the same script against
  multiple nREPL servers in one go.  Might be useful in ping-like queries.

## Miscallaneous to-do

- configure the CI to run the integration (Clojure) tests

## Details for selected features

### Argument handling and interpolation

- string/number/keyword magic by default
- `#nr[... :str]` and `#nr[... :clj]` directives
- `#nr[... :name <argname>]` → human readable `<argname>` in `--help` synopsis
- `#nr[... :help "<description>"]` → argument description in `--help`
- `#nr[... :or <value>]` for providing a default value
- `#nr[<arg> :or-void]` for producing **nothing**, if `<arg>` not provided
  (otherwise `nil` or `""`)
- `#nr[... :env <varname>]` for capturing the value from the environment
- **Ask argument values interactively**: When running in terminal context (tty)
  the program could ask the user to input values for arguments that are missing
  them.

- provide tagged literal function `#nr` for running the scripts directly on the
  host Clojure process (for testing purposes)
