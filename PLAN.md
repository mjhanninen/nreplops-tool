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

## Outstanding known bugs

None 😎

## nREPL session features

- `^C` interrupts evaluation (now: runaway)
- Implement timeout; we already have `--timeout` option without implementation, related to `^C`
- Return a distinct non-zero error code if evaluation throws
  - option to opt-out
- Short-circuiting:
  - given `-e A -e B` when `A` fails don't evaluate `B`
  - prerequisites: source parsing
  - option to opt-out

## Connection features

- Hostname assertions for SSH connections
- Merge `--port` and `--port-file` options. `--port` already handles host
  aliases, so why not port file paths
- Host name assertion for SSH: safer, esp. with port forwarding vpns
- Check that SSH client is present and one that we support
- Cache successfully resolved routes for SSH: reduces unnecesary knocking
- Reuse tunneled connection (see OpenSSH ControlMaster)
- **Skip SSH fingerprint**: When tunneling through a stable localhost port (can
  happen with VPN setups, for example) to multiple different remote hosts you
  easily end up in a sitation in which the host fingerprint check fails.  Add an
  option to host configuration (`nreplops-hosts.toml`) to allow skipping this
  check.  (Effectively `-o StrictHostKeyChecking=no`.)  Alternatively document
  how to achieve the same by changing `~/.ssh/config`.
- Support [socket prepl](./notes/prepl.md)
- **Run against multiple servers**: Enable running the same script against
  multiple nREPL servers in one go.  Might be useful in ping-like queries.

## Scripting/shebang features

- Proper argument handling and interpolation (see below)
- `--help` renders script's help when invoked via shebang
- Tooling for rendering script "docstrings" into Markdown
- `--production` with optional "are you sure?" mechanism
- `--dry-run` for debugging (combine with `--exprs-to` to see what would be sent)
- Creating and editing scripts (see [Subcommands](#subcommands) below)
- Checking code and docs parse okay (see [Subcommands](#subcommands) below)
- `--watch` the script and input files and resubmit upon change
- Rewrite rules (e.g `clojure.pprint/print` → `puget.cprint`)
  - conditional on stdout or stderr being connected to local tty
  - note: pointless, if we pretty-print ourselves

## Output features

- `--exprs-to <sink>` for echoing sent expressions to `<sink>`
- `--stdin-to <sink>` for echoing sent input to `<sink>`
- `--log-to <sink>` write an execution log to a file
- `--log` write an execution log to a file named by the source file
- JSON encoded output
- YAML encoded output
- Table output

## Other features

- Windows support

## Miscallaneous to-dos

- Fix bad error message "bad source file": What does it mean? Which file?
- Configure the CI to run the integration (Clojure) tests

## More details on selected features

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

### Subcommands

The subcommands could be:

- `exec`: Send the forms to nREPL server for evaluation.  This is what the
  program currently does.  Should be the default in the sense that if no
  subcommand is given then `exec` is assumed.
- `check`: Checks that the given input files can be parsed by the program.
  Covers both the code and the script documentation that has its own special
  comment format
- `edit`: Opens or creates the given files for editing in the user's default
  editor.  Furnishes the file, in case it is newly created, with shebang spell
  and documentation template.
- `pp` or `pretty`: Pretty prints the input.
