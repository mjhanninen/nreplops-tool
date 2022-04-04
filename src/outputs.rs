// outputs.rs
// Copyright 2022 Matti Hänninen
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not
// use this file except in compliance with the License. You may obtain a copy of
// the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations under
// the License.

use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
    fmt, fs,
    io::{self, Write},
    path,
    rc::Rc,
};

use crate::{
    cli::{self, IoArg},
    error::Error,
};

#[derive(Clone)]
pub enum Output {
    StdOut,
    StdErr,
    File(Rc<RefCell<io::BufWriter<fs::File>>>),
}

impl Output {
    pub fn writer(&self) -> OutputWriter<'_> {
        match *self {
            Output::StdOut => OutputWriter::StdOut,
            Output::StdErr => OutputWriter::StdErr,
            Output::File(ref f) => OutputWriter::File(f.borrow_mut()),
        }
    }
}

pub enum OutputWriter<'a> {
    StdOut,
    StdErr,
    File(RefMut<'a, io::BufWriter<fs::File>>),
}

impl<'a> Write for OutputWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match *self {
            OutputWriter::StdOut => io::stdout().lock().write(buf),
            OutputWriter::StdErr => io::stderr().lock().write(buf),
            OutputWriter::File(ref mut f) => f.write(buf),
        }
    }

    fn write_fmt(&mut self, fmt: fmt::Arguments) -> io::Result<()> {
        match *self {
            OutputWriter::StdOut => io::stdout().lock().write_fmt(fmt),
            OutputWriter::StdErr => io::stderr().lock().write_fmt(fmt),
            OutputWriter::File(ref mut f) => f.write_fmt(fmt),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match *self {
            OutputWriter::StdOut => io::stdout().lock().flush(),
            OutputWriter::StdErr => io::stderr().lock().flush(),
            OutputWriter::File(ref mut f) => f.flush(),
        }
    }
}

pub struct Outputs {
    // Receives our "internal" normal output
    pub stdout: Output,
    // Receives our "internal" error output
    pub stderr: Output,
    // Receives nREPL's standard output
    pub nrepl_stdout: Option<Output>,
    // Receives nREPL's standard error
    pub nrepl_stderr: Option<Output>,
    // Receives result forms
    pub nrepl_results: Option<Output>,
}

impl Outputs {
    pub fn try_from_args<'b>(args: &'b cli::Args) -> Result<Self, Error> {
        let mut logical_connections = HashMap::<Dst<'b>, Vec<Src>>::new();
        let mut connect = |source: Src, sink: Dst<'b>| {
            logical_connections
                .entry(sink)
                .or_insert_with(Vec::new)
                .push(source);
        };
        match args.stdout_to {
            Some(IoArg::Pipe) => connect(Src::StdOut, Dst::StdOut),
            Some(IoArg::File(ref p)) => connect(Src::StdOut, Dst::File(p)),
            None => (),
        }
        match args.stderr_to {
            Some(IoArg::Pipe) => connect(Src::StdErr, Dst::StdErr),
            Some(IoArg::File(ref p)) => connect(Src::StdErr, Dst::File(p)),
            None => (),
        }
        match args.results_to {
            Some(IoArg::Pipe) => connect(Src::Results, Dst::StdOut),
            Some(IoArg::File(ref p)) => connect(Src::Results, Dst::File(p)),
            None => (),
        }
        let stdout = Output::StdOut;
        let stderr = Output::StdErr;
        let mut nrepl_stdout = None;
        let mut nrepl_stderr = None;
        let mut nrepl_results = None;
        for (sink, sources) in logical_connections.into_iter() {
            let output = match sink {
                Dst::StdOut => stdout.clone(),
                Dst::StdErr => stderr.clone(),
                Dst::File(ref p) => {
                    let f = fs::File::create(p).map_err(|_| {
                        Error::CannotWriteFile(p.to_string_lossy().to_string())
                    })?;
                    let w = io::BufWriter::new(f);
                    Output::File(Rc::new(RefCell::new(w)))
                }
            };
            for source in sources.into_iter() {
                match source {
                    Src::StdOut => nrepl_stdout = Some(output.clone()),
                    Src::StdErr => nrepl_stderr = Some(output.clone()),
                    Src::Results => nrepl_results = Some(output.clone()),
                }
            }
        }
        Ok(Self {
            stdout,
            stderr,
            nrepl_stdout,
            nrepl_stderr,
            nrepl_results,
        })
    }
}

// Sources of output on the nREPL's side
#[derive(Debug)]
enum Src {
    StdOut,
    StdErr,
    Results,
}

// Local destinations for output
#[derive(Debug, PartialEq, Eq, Hash)]
enum Dst<'a> {
    StdOut,
    StdErr,
    File(&'a path::Path),
}
