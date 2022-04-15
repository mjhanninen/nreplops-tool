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
