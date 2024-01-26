// pprint/printer.rs
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

use std::io::{self, Write};

use super::{fragments::Fragment, style::Style};

pub enum Command<'a> {
  NewLine,
  Space(u16),
  SetStyle(Style),
  ResetStyle(Style),
  Text(&'a str),
}

impl<'a> Command<'a> {
  fn is_set_style(&self) -> bool {
    matches!(self, Command::SetStyle(_))
  }
}

const SPACES_LEN: usize = 64;
static SPACES: [u8; SPACES_LEN] = [b' '; SPACES_LEN];

pub fn print<'a, W, I>(
  writer: &mut W,
  input: I,
  use_color: bool,
) -> io::Result<()>
where
  W: Write,
  I: Iterator<Item = &'a Command<'a>>,
{
  use Command as C;

  let mut current_color = None;
  let mut it = input.peekable();

  while let Some(command) = it.next() {
    match command {
      C::NewLine => writeln!(writer)?,
      C::Space(amount) => {
        let mut remaining = *amount as usize;
        while remaining > SPACES_LEN {
          writer.write_all(&SPACES)?;
          remaining -= SPACES_LEN;
        }
        if remaining > 0 {
          writer.write_all(&SPACES[0..remaining])?;
        }
      }
      C::SetStyle(style) => {
        if use_color {
          let new_color = style.to_ansi_color();
          match current_color {
            Some(old_color) if old_color == new_color => (),
            _ => {
              write!(writer, "{}", new_color.render_fg())?;
              current_color = Some(new_color);
            }
          }
        }
      }
      C::ResetStyle(style) => {
        if use_color
          && it.peek().map(|next| !next.is_set_style()).unwrap_or(true)
        {
          write!(writer, "{}", anstyle::Reset.render())?;
          current_color = None;
        }
      }
      C::Text(text) => writer.write_all(text.as_bytes())?,
    }
  }
  Ok(())
}

pub trait BuildInput<'a> {
  fn add_new_line(&mut self);
  fn add_spaces(&mut self, amount: u16);
  fn add_fragment(&mut self, fragment: &'a Fragment<'a>);
}

impl<'a> BuildInput<'a> for Vec<Command<'a>> {
  fn add_new_line(&mut self) {
    self.push(Command::NewLine);
  }
  fn add_spaces(&mut self, amount: u16) {
    self.push(Command::Space(amount))
  }
  fn add_fragment(&mut self, fragment: &'a Fragment<'a>) {
    self.push(Command::SetStyle(fragment.style));
    self.push(Command::Text(fragment.text.as_str()));
    self.push(Command::ResetStyle(fragment.style));
  }
}
