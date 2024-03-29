---
title: nr
section: 1
header: nREPL Ops Tool Manual
---

# NAME

nr - The nREPL ops tool

# SYNOPSIS

| **nr** \[_options_] _file_ \[_args_]
| **nr** \[_options_] **-f** _file_ ...\ \[_args_]
| **nr** \[_options_] **-e** _expr_ ...\ \[_args_]
| **nr** \[_options_] **-!**\ \[_version-requirement_] _file_ \[_args_]
| **nr** **\--wait-port-file** _seconds_
| **nr** **\--version**
| **nr** \[**-h**|**\--help**]

# OPTIONS

## General options

**-!** \[_version-requirement_]

:   Activates shebang mode, altering the tool's behavior in certain ways.

    This flag allows for an optional version requirement to ensure the tool's
    version complies.

    The version requirement can be a single version, where the tool's version
    must be at least that version, adhering to semantic versioning principles.
    For major version 0, minor version updates are considered breaking. The
    requirement can also be an explicit version range, with the end version
    excluded.

**\--timeout** _seconds_

:   Aborts the program execution after _seconds_ have elapsed unless the program
    has managed to complete before it.

    The duration is measured from the very start of the program execution and
    includes, for example, the time elapsed while waiting for the port file to
    appear (see the **\--wait-port-file** option).

**-V**, **\--version**

:   Prints the version information.

## Connection options

**-p**, **\--port**, **\--host** \[\[_tunnel_:]_host_:]_port_

:   Connects to the nREPL server listening on the \[_host_:]_port_.

    The _host_, if given, can be an IPv4 address, IPv6 address, or domain name.
    In case the domain name resolves to multiple addresses the IPv4 addresses
    are preferred over the IPv6 addresses.

    The _tunnel_, if given, should be of the form
    \[_login_@]_ssh-host_\[:_ssh-port_] specifying the SSH connection through
    which the nREPL connection is to be tunneled.  When the connection is
    tunneled the name and address resolution of the _host_ happens on the
    forwarding SSH host.  Tunneling requires that the local system has the
    OpenSSH remote login client (ssh) installed on it.

    If this option is not given then the program searches for a `.nrepl-port`
    file and reads the connection information from it.  The search covers the
    current working directory and its ancestors and the nearest matching file
    is selected.

    See also the **\--port-file** option.

**\--port-file** _file_

:   Reads the nREPL server connection information from the given _file_ instead
    of searching for the nearest `.nrepl-port` file.

    The **\--port** option, if given, takes precedence over this option.

**\--wait-port-file** _seconds_

:   Waits _seconds_ for the port file to become available if none exists when
    the program starts.  After _seconds_ have elapsed the program aborts
    execution with the timeout status unless the port file has become available.

    This option can be used without supplying the program any expressions to be
    evaluted the server.  In that case the program just waits for the port file
    to appear and then returns immediately.

## Evaluation options

**-a**, **\--arg** _name=value_

:   Sets the template argument _name_ to _value_.

    **NB:** Currently _value_ is interpolated into the source code as-is without
    any kind of interpretation.  For example, in order to pass a string you need
    to pass the double quotes with string:

    ```
    nr --arg 'foo="Hello world"'
    ```

    However this behavior will change in future versions.

**-e**, **\--expr** _expression_

:   Evaluates the _expression_ on the nREPL server.

    This option can be given multiple times in which case all expressions are
    evaluated within the same nREPL session in the left-to-right order.

    This option conflicts with the **\--file** option.

**-f**, **\--file** _file_

:   Evaluates the whole content of the _file_ on the nREPL server.  The file can
    contain more than one expression.

    This option can be given multiple times in which case the files are
    evaluated within the same nREPL session in the order they are given.  For
    example:

    ```
    nr -f first.clj -f second.clj
    ```

    This option conflicts with the **\--expr** option.

**\--ns**, **\--namespace** _namespace_

:   Evaluates the expressions within the _namespace_.

    If this option is not given then the expressions are evaluated within the
    `*user*` namespace.

## Input, output, and result options

**\--in**, **\--input**, **\--stdin** _file_

:   Sends the content of _file_ to the nREPL server as the remote standard input.

    If _file_ is `-` then the local standard input is forwarded to the nREPL
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

**\--stdout**, **\--out**, **\--output** _file_

:   Writes the nREPL server's standard output to _file_.

    If this option is not given then the remote output is directed to the local
    standard output.

    See also the **\--no-stdout** option.

**\--no-stdout**, **\--no-out**, **\--no-output**

:   Discards the nREPL server's standard output.

    This option conflicts with the **\--out** option.

**\--stderr**, **\--err** _file_

:   Writes the nREPL server's standard serror to _file_.

    If this option is not given then the remote output is directed to the local
    standard error.

    See also the **\--no-stderr** option.

**\--no-stderr**, **\--no-err**

:   Discards the nREPL server's standard error.

    This option conflicts with the **\--stderr** option.

**\--results**, **\--res**, **\--values** _file_

:   Writes the evaluation results to _file_, a single result per line.

    If this option is not given then the results are directed to the local
    standard output.

    See also the **\--no-results** option.

**\--no-results**, **\--no-res**, **\--no-values**

:   Discards evaluation results.  This can be useful when the expressions are
    evaluated only for their side-effects.

    This option conflicts with the **\--results** option.

**\--pretty**, **\--no-pretty**

:   Controls pretty-printing of evaluation results. By default, output is
    pretty-printed to the terminal and unformatted for pipes or files.

**\--color**, **\--no-color**

:   Controls output colorization. By default, output is colored for terminal and
    plain for pipes or files.

# EXIT STATUS

An exit status of zero indicates success and a non-zero status indicates
failure. The possible exit status codes are the following:

| Status | Reason  |
|:-------|:--------|
| 0      | Success |
| 1      | Error   |
| 2      | Timeout |
