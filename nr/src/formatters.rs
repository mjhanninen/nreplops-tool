use std::io::{self, Write};

use crate::clojure::lex::Lexeme;

#[derive(Debug)]
pub struct Formatter {
  pub pretty: bool,
  pub color: bool,
}

impl Formatter {
  pub fn new(pretty: bool, color: bool) -> Self {
    Self { pretty, color }
  }

  pub fn write(
    &self,
    writer: &mut impl Write,
    lexemes: &[Lexeme],
  ) -> io::Result<()> {
    writeln!(writer, "{:?}", lexemes)
  }
}
