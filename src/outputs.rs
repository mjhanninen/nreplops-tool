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
  path::{self, Path},
  rc::Rc,
};

use crate::{
  cli::{self, IoArg},
  error::Error,
};

#[derive(Clone, Debug)]
pub enum Output {
  StdOut,
  StdErr,
  File {
    file: Rc<RefCell<io::BufWriter<fs::File>>>,
    path: Box<Path>,
  },
}

impl Output {
  pub fn writer(&self) -> OutputWriter<'_> {
    match *self {
      Output::StdOut => OutputWriter::StdOut,
      Output::StdErr => OutputWriter::StdErr,
      Output::File { ref file, ref path } => OutputWriter::File {
        file: file.borrow_mut(),
        path,
      },
    }
  }
}

#[derive(Debug)]
pub enum OutputWriter<'a> {
  StdOut,
  StdErr,
  File {
    file: RefMut<'a, io::BufWriter<fs::File>>,
    path: &'a Path,
  },
}

impl Output {
  pub fn target(&self) -> OutputTarget {
    match self {
      Output::StdOut => OutputTarget::StdOut,
      Output::StdErr => OutputTarget::StdErr,
      Output::File { ref path, .. } => OutputTarget::File(path),
    }
  }
}

impl<'a> Write for OutputWriter<'a> {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    match *self {
      OutputWriter::StdOut => io::stdout().lock().write(buf),
      OutputWriter::StdErr => io::stderr().lock().write(buf),
      OutputWriter::File { ref mut file, .. } => file.write(buf),
    }
  }

  fn write_fmt(&mut self, fmt: fmt::Arguments) -> io::Result<()> {
    match *self {
      OutputWriter::StdOut => io::stdout().lock().write_fmt(fmt),
      OutputWriter::StdErr => io::stderr().lock().write_fmt(fmt),
      OutputWriter::File { ref mut file, .. } => file.write_fmt(fmt),
    }
  }

  fn flush(&mut self) -> io::Result<()> {
    match *self {
      OutputWriter::StdOut => io::stdout().lock().flush(),
      OutputWriter::StdErr => io::stderr().lock().flush(),
      OutputWriter::File { ref mut file, .. } => file.flush(),
    }
  }
}

#[derive(Debug)]
pub enum OutputTarget<'a> {
  StdOut,
  StdErr,
  File(&'a Path),
}

#[derive(Debug)]
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
  pub fn try_from_args(args: &cli::Args) -> Result<Self, Error> {
    fn err_cannot_write_file(path: &path::Path) -> Error {
      Error::CannotWriteFile(path.to_string_lossy().to_string())
    }

    fn file_dst_from_path(path: &path::Path) -> Result<Dst, Error> {
      path
        .canonicalize()
        .map(|p| Dst::File(p.into()))
        .map_err(|_| err_cannot_write_file(path))
    }

    let mut logical_connections = HashMap::<Dst, Vec<Src>>::new();

    let mut connect = |source: Src, sink: Dst| {
      logical_connections
        .entry(sink)
        .or_insert_with(Vec::new)
        .push(source);
    };

    match args.stdout_to {
      Some(IoArg::Pipe) => connect(Src::StdOut, Dst::StdOut),
      Some(IoArg::File(ref p)) => connect(Src::StdOut, file_dst_from_path(p)?),
      None => (),
    }
    match args.stderr_to {
      Some(IoArg::Pipe) => connect(Src::StdErr, Dst::StdErr),
      Some(IoArg::File(ref p)) => connect(Src::StdErr, file_dst_from_path(p)?),
      None => (),
    }
    match args.results_to {
      Some(IoArg::Pipe) => connect(Src::Results, Dst::StdOut),
      Some(IoArg::File(ref p)) => connect(Src::Results, file_dst_from_path(p)?),
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
        Dst::File(path) => {
          let f = fs::File::create(&path)
            .map_err(|_| err_cannot_write_file(&path))?;
          let w = io::BufWriter::new(f);
          Output::File {
            file: Rc::new(RefCell::new(w)),
            path,
          }
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
enum Dst {
  StdOut,
  StdErr,
  File(Box<Path>),
}
