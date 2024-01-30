// pprint/mod.rs
// Copyright 2024 Matti Hänninen
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

//
// So the different representations that the result being printed passes through
// are the following:
//
// 1. unparsed result
// 2. list of lexemes
// 3. result value AST
// 4. input structure to layout solver
// 5. input structure to printer
//
// The unparsed result (1) is what we receive from the nREPL server.  This
// is produced by Clojure's printer so it is Clojure but, importantly, very
// restricted form of Clojure.  It does not contain meta data literal and most
// of the reader macros are absent as well.
//
// The lexemes (2) are produced with our Clojure lexer and should be able to
// represent the whole language.  However, as the input result is restricted
// form of Clojure, so the produced lexemes are also a subset of known lexemes.
//
// The abstract syntax tree (3) is capable of representing only a limited subset
// of Clojure; this is effectively the EDN.  It is rich enough so that we can
// formulate the problem for the layout solver and, optionally, translate the
// data into some other form (e.g. JSON, YAML, or CSV).
//
// The input to the pretty-printing layout solver (4) is either a flat list
// or a tree structure.  I'm not sure which one it will be.  In any case it
// conssists of chunks of unbreakable texts together with styling information,
// suggested breakpoints, layout anchors, optional whitespacing, relationships
// between breakpoints and the like.  Things that the layout solver uses while
// determining the optimal layout.
//
// The input to the printer (5) consists of text fragments, style coding
// (separated from the text), line breaks, and spacing.  There is not much
// conditionality at this phase; only the decision whether to include or
// ignore the styling (coloring) when printing according to the alraedy fixed
// layout.
//
// When we are printing unformatted but colored output we can skip the phases
// (3) and (4) produce the printer input (5) directly from the lexemes (2).
//
// When we are converting the results to some other output format (e.g. JSON,
// YAML, or CSV) we follow through the same phases but do the translation
// into the alternative output format when we produce the input for the layout
// solver.
//

use std::io::{self, Write};

mod fragments;
mod layout_solver;
mod pretty_edn;
mod printer;
mod style;
mod unformatted_edn;

use crate::clojure::{lex::Lexeme, result_ir};

#[derive(Debug)]
pub struct ClojureResultPrinter {
  pub pretty: bool,
  pub color: bool,
  pub width: u16,
}

impl ClojureResultPrinter {
  pub fn new(pretty: bool, color: bool, width: u16) -> Self {
    Self {
      pretty,
      color,
      width,
    }
  }

  pub fn print(
    &self,
    writer: &mut impl Write,
    lexemes: &[Lexeme],
  ) -> io::Result<()> {
    let mut printer_input = Vec::new();
    if self.pretty {
      let value = result_ir::build(lexemes).unwrap();
      let chunks = pretty_edn::convert_to_layout_program(&value);
      layout_solver::solve(&chunks, &mut printer_input);
    } else {
      unformatted_edn::generate_printer_input(
        lexemes.iter(),
        &mut printer_input,
      );
    }
    printer::print(writer, printer_input.iter(), self.color)?;
    writeln!(writer)
  }
}
