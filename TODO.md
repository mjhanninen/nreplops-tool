# To-Do

- configure the CI to run the integration (Clojure) tests
- `--wait-port-file <timeout>` option
- `--exprs-to <sink>` option echos sent expressions to `<sink>`
- `--stdin-to <sink>` option echos sent input to `<sink>`
- short-circuit: given `-e A -e B` if `A` fails, don't evaluate `B`
- interpolation with string/number magic
- interpolation directives (e.g. :s for strings)
- values from environment variables
- JSON encoded results (pipe to jq)
- tagged literal functions for running the scripts directly on the host Clojure process
- `--help` renders script help when invoked via shebang
- `-!` takes minimum version
- `#nr[... :name <argname>]` → human readable `<argname>` in `--help` synopsis
- `#nr[... :or <value>]` for providing a default value
- `#nr[<arg> :or-void]` for having nothing, if `<arg>` not provided
- replacements (e.g clojure.pprint/print → puget.cprint) when sending stdout to terminal
- tunnel through ssh (libssh or libssh2)
