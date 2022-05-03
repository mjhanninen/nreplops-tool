# To-Do

## Features

- **Check SSH client**: Check that the SSH client is present and one that we
  support.
- **Timeout**: We already have the `--timeout` option for this. It just misses
  the implementation.
- **Cache successful route**: When a connection expression resolves to multiple
  routes cache the one that succeed.  The point is to reduce unnecessary
  knocking.
- argument interpolation
  - string/number/keyword magic by default
  - `#nr[... :str]` and `#nr[... :clj]` directives
  - `#nr[... :name <argname>]` → human readable `<argname>` in `--help` synopsis
  - `#nr[... :help "<description>"]` → argument description in `--help`
  - `#nr[... :or <value>]` for providing a default value
  - `#nr[<arg> :or-void]` for producing **nothing**, if `<arg>` not provided
    (otherwise `nil` or `""`)
  - `#nr[... :env <varname>]` for capturing the value from the environment
- `--help` renders script help when invoked via shebang
- JSON encoded results (for piping to `jq`)
- [socket prepl support](./prepl.md)
- browser repl support (find a way to browser no matter what)
- configure the CI to run the integration (Clojure) tests
- short-circuiting: given `-e A -e B` when `A` fails don't evaluate `B`
- `--exprs-to <sink>` for echoing sent expressions to `<sink>`
- `--stdin-to <sink>` for echoing sent input to `<sink>`
- `-!` takes minimum version
- `--dry-run` for debugging (combine with `--exprs-to` to see what would be sent)
- rewrite rules (e.g `clojure.pprint/print` → `puget.cprint`)
  - conditional on stdout or stderr being connected to tty locally
- **Watch and resubmit on change**: Watch the script and input files and send
  the expressions for evaluation on observing a change. Could be enabled with
  the `--watch` option.
- **Ask confirmation in production**: Add a flag to the host configuration
  (`nreplops-hosts.toml`) that allows indicating a host as a production host and
  allow the user to review the sent expression and confirm it before sending it
  away.
- **Ask argument values interactively**: When running in terminal context (tty)
  the program could ask the user to input values for arguments that are missing
  them.
- **Skip SSH fingerprint**: When tunneling through a stable localhost port (can
  happen with VPN setups, for example) to multiple different remote hosts you
  easily end up in a sitation in which the host fingerprint check fails.  Add an
  option to host configuration (`nreplops-hosts.toml`) to allow skipping this
  check.  (Effectively `-o StrictHostKeyChecking=no`.)  Alternatively document
  how to achieve the same by changing `~/.ssh/config`.
- **Run against multiple servers**: Enable running the same script against
  multiple nREPL servers in one go.  Might be useful in ping-like queries.
- provide tagged literal function `#nr` for running the scripts directly on the
  host Clojure process (for testing purposes)
- Windows support

## Improving and fixing things

- **Error handling**: So far the error handling has been more of an
  afterthought.  Start using `error::Error` consistently everywhere except the
  very few special places where the error value is meaningful for the control
  flow (e.g. in the Bencode scanning).  Get rid of `anyhow::Error`.

## Documentation

- add a connection examples section to the manual
