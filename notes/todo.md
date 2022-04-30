# To-Do

## Features

- check the presence and version of the ssh client as needed (ssh)
- don't let other things block the timeout watchdog
- when a port expression resolves to multiple routes, cache the one that succeed
- argument interpolation
  - string/number/keyword magic by default
  - `#nr[... :str]` and `#nr[... :clj]` directives
  - `#nr[... :name <argname>]` → human readable `<argname>` in `--help` synopsis
  - `#nr[... :help "<description>"]` → argument description in `--help`
  - `#nr[... :or <value>]` for providing a default value
  - `#nr[<arg> :or-void]` for producing **nothing**, if `<arg>` not provided (otherwise `nil` or `""`)
  - `#nr[... :env <varname>]` for capturing the value from the environment
- `--help` renders script help when invoked via shebang
- JSON encoded results (for piping to `jq`)
- socket REPL support
- pREPL support
- browser repl support (find a way to browser no matter what)
- configure the CI to run the integration (Clojure) tests
- short-circuiting: given `-e A -e B` when `A` fails don't evaluate `B`
- `--exprs-to <sink>` for echoing sent expressions to `<sink>`
- `--stdin-to <sink>` for echoing sent input to `<sink>`
- `-!` takes minimum version
- `--dry-run` for debugging (combine with `--exprs-to` to see what would be sent)
- rewrite rules (e.g `clojure.pprint/print` → `puget.cprint`)
  - conditional on stdout or stderr being connected to tty locally
- Windows support
- provide tagged literal function `#nr` for running the scripts directly on the host Clojure process (for testing purposes)

## Documentation

- add a connection examples section to the manual
