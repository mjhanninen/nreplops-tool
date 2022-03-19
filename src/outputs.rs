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
    fs,
    io::{self, Write},
    path,
    rc::Rc,
};

use crate::cli::{self, IoArg};

#[derive(Debug, thiserror::Error)]
pub enum Error {}

#[derive(Clone, Default)]
pub struct Output<'a>(Option<Rc<RefCell<dyn Write + 'a>>>);

impl<'a> Output<'a> {
    pub fn new(w: impl Write + 'a) -> Self {
        Self(Some(Rc::new(RefCell::new(w))))
    }

    // Panics (by design) if borrowed in two places at the same time
    pub fn borrow_mut(&self) -> Option<RefMut<'_, (dyn Write + 'a)>> {
        self.0.as_ref().map(|r| r.borrow_mut())
    }
}

// XXX(soija) Fix Output's interior to Rc<RefCell<...>> and these to
//            Option<Output<'a>>. Also implement Write for Output.
#[derive(Default)]
pub struct Outputs<'a> {
    // Receives nREPL's standard output
    pub stdout: Output<'a>,
    // Receives nREPL's standard error
    pub stderr: Output<'a>,
    // Receives result forms
    pub results: Output<'a>,
}

impl<'a> Outputs<'a> {
    pub fn try_from_args<'b>(args: &'b cli::Args) -> Result<Self, Error> {
        let mut logical_connections = HashMap::<Dst<'a>, Vec<Src>>::new();
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
        let mut self_ = Self::default();
        for (sink, sources) in logical_connections.into_iter() {
            let output = match sink {
                Dst::StdOut => Output::new(io::stdout().lock()),
                Dst::StdErr => Output::new(io::stderr().lock()),
                Dst::File(ref p) => {
                    let f = fs::File::create(p).unwrap();
                    let w = io::BufWriter::new(f);
                    Output::new(w)
                }
            };
            for source in sources.into_iter() {
                match source {
                    Src::StdOut => self_.stdout = output.clone(),
                    Src::StdErr => self_.stderr = output.clone(),
                    Src::Results => self_.results = output.clone(),
                }
            }
        }
        Ok(self_)
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
