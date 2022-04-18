# nreplops-tool (nr)

nreplops-tool (`nr`) is a non-interactive nREPL client designed to be used in
shell scripts and on the command-line.

> **Early α warning:**  This project is still at a very early development
> phase.  While the tool is probably reasonably usable (and the author uses it
> daily) the current development focus is not on quality, stability, or
> documentation.

## Try it out in 1 minute

This example assumes you are able to install packages with Homebrew.  See the
[Installation](#installation) section below for other options.

Start by installing nreplops-tool (`nr`) and Babashka (`bb`):

```
brew install mjhanninen/sour/nreplops-tool borkdude/brew/babashka
```

Launch a Babashka nREPL server (that listens on the port 1667 by default):

```
bb nrepl-server
```

Open another terminal and evaluate an expression with `nr`:

```
nr -p 1667 -e '(println "Hello, world!")'
```

## Quick examples

Before starting make sure that you have a Clojure nREPL server running in the
background and there is a corresponding `.nrepl-port` file in either the current
working directory or any of its ancestor.

Evaluate the expression `(+ 1 2)` on a nREPL server:

```
$ nr -e '(+ 1 2)'
3
```

Pass the expressions through a pipe:

```
$ echo '(+ 1 2)' | nr
3
```

Evaluate the content of a file:

```
$ echo '(+ 1 2)' > plus.clj
$ nr plus.clj
3
```

Create an executable nREPL scripts:

```
$ cat <<EOF > plus.nrepl
+ #!/usr/bin/env nr -!
+ (+ 1 2)
+ EOF
$ chmod +x plus.nrepl
$ ./plus.nrepl
3
```

Suppose the nREPL server had a function called `get-user-by-email` that searched
in retrieved users from the application database by email.  A script exposing
that functionality to the command line could look something like this:

```
$ cat <<EOF > get-user-by-email.nrepl
+ #!/usr/bin/env nr -! --no-results
+ (clojure.pprint/pprint
+   (get-user-by-email db "#nr[1]"))
$ EOF
$ chmod +x get-user-by-email.nrepl
$ ./get-user-by-email.nrepl wile.e.coyote@example.com
{:name "Wile E. Coyote"
 :email "wile.e.coyote@example.com"
 :phone "555-555 555"}
```

## Installation

### Homebrew

```
brew install mjhanninen/sour/nreplops-tool
```

### Cargo

```
cargo install nreplops-tool
```

### Building from sources

```
git clone https://github.com/mjhanninen/nreplops-tool.git
cargo install --path .
```

## Goals

- Easy of use in shell scripts
- Consistency and interoperability with Babashka and jq

## License

Copyright 2022 Matti Hänninen

Licensed under the Apache License 2.0

Please see the [LICENSE](./LICENSE) and [NOTICE](./NOTICE) files.
