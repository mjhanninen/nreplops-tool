---
title: nr
section: 1
header: nREPL Ops Tool Manual
---

# NAME

nr - The nREPL ops tool

# SYNOPSIS

| **nr** \[_options_] _file_ \[_args_]
| **nr** \[_options_] **-f** _file_ `...` \[_args_]
| **nr** \[_options_] **-e** _expr_ `...` \[_args_]
| **nr** \[_options_] **-!** _file_ \[_args_]
| **nr** **\--wait-port-file** _seconds_
| **nr** **\--version**
| **nr** \[**-h**|**\--help**]

# DESCRIPTION

(Rewrite the description; the following is just a verbatim copy of the
docstring for the positional arguments.)

Positional arguments whose interpretation depends on the presence and absence
of different options.

Normally the first argument gives the source file and any remaining arguments
are interpreted as positional template arguments to be interpolated into the
source.  The source file argument is interpreted like the **\--file** option.

If either **\--file** or **\--expr** options are present or the command is run
within a pipe then all positional arguments are interpreted as values for
template variables.

The template variable values given through **\--arg** options are prepended to
the positional arguments  present then the positional arguments that

# OPTIONS

## General options

**-!**
:   Run in the shebang mode.

    Allows only those options and arguments that are safe to use while running
    within a shebang context (i.e. when invoked through `#!`).

## Connection options

**-p**, **\--port** \[_host_:]_port_

:   Connects to the nREPL server listening on the \[_host_:]_port_.

    The _host_, if given, can be an IPv4 address, IPv6 address, or a domain
    name.  The domain name resolution prefers IPv4 addresses over IPv6
    addresses in case the name resolves to multiple addresses.

    If this option is not given then the program consults the nearest
    `.nrepl-port` file for the connection info.  The file is searched from the
    current working directory and its ancestors.

    See also the **\--port-file** option.

**\--port-file** _file_

:   Reads the nREPL server connection info from the _file_ instead of searching
    for the nearest `.nrepl-port` file.

    The **\--port** option, if given, takes precedence over this option.

**\--wait-port-file** _seconds_

:   Waits given _seconds_ for the port file to appear before attempting to
    connect to the server.

    In case of a timeout the program aborts with a non-zero exit code.

    This option can be given without supplying any expression input.  In that
    case the program just waits for the port file and, upon success, returns
    immediately with the exit code 0.

## Evaluation options

**-a**, **\--arg** _name=value_

:   Set the template argument _name_ to _value_.

    The template arguments are textual. If they look like a number they are
    interpolated as a number and otherwise they are interpolated as a string
    literal.

**-e**, **\--expr** _expression_

:   Evaluates the _expression_ on the nREPL server.

    This option can be given multiple times in which case all expressions are
    evaluated within the same nREPL session in the left-to-right order.

    This option conflicts with the **\--file** option.

**-f**, **\--file** _file_

:   Evaluates the whole content of the _file_ on the nREPL server.  The file
    can contain more than one expression.

    This option can be given multiple times in which case the files are
    evaluated within the same nREPL session in the left-to-right order.  For
    example:

    ```
    nr -f first.clj -f second.clj
    ```

    This option conflicts with the **\--expr** option.

**\--ns**, **\--namespace** _namespace_

:   Evaluates the expressions within the _namespace_.

## Result and output options

**\--stdin** _file_

:   Sends the content of _file_ to the nREPL server as the remote standard input.

    If _file_ is `-` then the local standard input is tunneled to the nREPL
    server.  This requires the use of either **\--expr** or **\--file** option
    to pass the expressions.  For example:

    ```
    $ echo '"World"' \
        | nr --stdin - \
             --expr '(->> *in*
                          edn/read
                          (println "Hello,"))'
    Hello, World
    nil
    ```

    If this option is not given then nothing is sent over to the server's
    standard input.

**\--stdout** _file_

:   Writes the nREPL server's standard output to _file_. If not given then the
    remote output is directed to the local standard output.

**\--no-stdout**, **\--no-out**, **\--no-output**

:   Discards the nREPL server's standard output.

    This option conflicts with the **\--stdout** option.

**\--stderr** _file_

:   Writes the nREPL server's standard serror to _file_. If not given then the
    remote output is directed to the local standard error.

**\--no-stderr**, **\--no-err**, **\--no-error**

:   Discards the nREPL server's standard error.

    This option conflicts with the **\--stderr** option.

**\--res**, **\--results**, **\--values** _file_

:   Writes the evaluation results to _file_, a single result per line.  If not
    given then the results are directed to the local standard output.

**\--no-res**, **\--no-results**, **\--no-values**

:   Discards evaluation results. This can be useful when the expressions are
    evaluated only for their side-effects.

    This option conflicts with the **\--results** option.

# EXAMPLES

To be written.

# EXIT VALUES

0

: Success

