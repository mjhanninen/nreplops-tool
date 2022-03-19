# nreplops-tool (nr)

nreplops-tool is a non-interactive nREPL client designed to be used in shell
scripts and on the command line.

**WARNING:** This project is at **early concept** phase. There is a base
implementation to play with but it has rough edges and every new version will
probably be breaking for some time.

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
