# Planned features and goals

## Project goals

- Easy of use in shell scripts
- Consistency and interoperability with selected CLI tools: [Babashka]
  [babashka], [Jet][jet], and [jq][jq]

[babashka]: https://github.com/babashka/babashka
[jet]: https://github.com/borkdude/jet
[jq]: https://github.com/stedolan/jq

## Smaller features

- Return a distinct error code if evaluation throws; opt-in (now: 0)
- Stop evaluating forms after a throw; opt-in
- Write an execution log to a file; opt-in
- Capture arguments from environment (now: only from command line)
- Host name assertion (safer, esp. with port forwarding vpns)

## Larger features

- `^C` interrupts evaluation (now: runaway)
- Proper argument handling (now: text replacement)
- Reuse tunneled connection (see OpenSSH ControlMaster)
- Some kind of "are you sure?" safety mechanism
- Tooling for rendering script "docstrings" into Markdown

## Not 100% sure

- Merge `--port` and `--port-file` options
  - `--port` already handles host aliases, so why not port file paths
