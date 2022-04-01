# To-Do

- clojure.pprint/print to puget.cprint replacement when tty
- interpolation with string/number magic
- interpolation directives (e.g. :s for strings)
- values from environment variables
- JSON encoded results (pipe to jq)
- tagged literal functions for running the scripts directly on the host Clojure process
- `--help` renders script help when invoked via shebang
- `-!` takes minimum version
- `--wait-port-file <timeout>` option
- `#nr[... :name <argname>]` gives human readable `<argname>` in `--help`
- `#nr[... :or <value>]` for providing a default value
- `#nr[<arg> :or-void]` for having nothing, if `<arg>` not provided
- short-circuit: given `-e A -e B` if `A` fails, don't evaluate `B`
- `--exprs-to <sink>` option echos sent expressions to `<sink>`
- `--input-to <sink>` option echos sent input to `<sink>`
