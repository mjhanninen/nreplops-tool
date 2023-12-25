# nREPL Ops Tool

## Overview

> To be written; here's some ideas
>
> - a non-interactive nrepl client
> - shim between nrepl and other tools

## Features

- Works well with **jq**
- Works well with **Babashka**

## Usage scenarios

- Capture data from a remote Clojure host and post-process it with Babashka
  locally
- Write a command line scripts for querying things that you often need when
  debugging your system

## Table of contents

- Running `nr`
  - overview
  - command line options
  - exit status
- Using `nr` without SSH
- Using `nr` with Jq (`jq`)
- Using with Babashka (`bb`)
- REPL scripts
  - sample scripts

## Running `nr`

### Expression as command line argument

```
$ nr -e '(+ 1 2)'
3
```

### Expression through pipe

```
$ echo '(+ 1 2)' | nr
3
```

### Expressions in file

Create a file `plus.clj` with the following content:

```clojure
(+ 1 2)
```

Pass the file to `nr` through the command line:

```
$ nr plus.clj
3
```

### Expressions as script file

Create a script file `plus.nr.clj` with the following content:

```clojure
#!/usr/bin/env -S nr -!

(+ 1 2)
```

Ensure that the file is executable and then run it like so:

```
$ ./plus.nr.clj
3
```
