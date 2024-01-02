// main.rs
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

#![deny(
  future_incompatible,
  missing_debug_implementations,
  nonstandard_style,
  rust_2021_compatibility,
  // unused
)]
#![allow(unused)]

use std::{
  io::{self, Read},
  process,
};

use atty::Stream;
use nr_clj_lexer as lexer;

fn main() {
  if atty::is(Stream::Stdin) {
    eprintln!("ERROR: No input provided. Please provide input via stdin.");
    process::exit(1);
  }

  let mut input = String::new();
  if let Err(e) = io::stdin().read_to_string(&mut input) {
    eprintln!("ERROR: Failed to read from stdin: {}", e);
    process::exit(1);
  }

  match lexer::lex(&input) {
    Ok(lexemes) => {
      for (ix, lexeme) in lexemes.iter().enumerate() {
        println!("{}: {:?}", ix, lexeme);
      }
    }
    Err(e) => {
      eprintln!("ERROR: Failed to parse: {:?}", e);
      process::exit(1);
    }
  }
}
