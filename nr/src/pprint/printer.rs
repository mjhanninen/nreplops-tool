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

use std::{
  io::{self, Write},
  rc::Rc,
};

use super::{fragments::Fragment, style::Style};

pub enum Command {
  NewLine,
  Space(u16),
  SetStyle(Style),
  ResetStyle(Style),
  Text(Rc<str>),
}

impl Command {
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
  I: Iterator<Item = &'a Command>,
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
      // On console "having a style" means here "having a foreground color from
      // the ANSI 16 palette".  Hence, we don't bother to check which and what
      // kind of span we are closing.  Instead we just check if the following
      // text has "style" as well and, if so, don't bother even resetting the
      // current one.  If we were generating HTML, we would probably want to
      // emit a correct closing tag here.
      C::ResetStyle(_style) => {
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

pub trait BuildInput {
  fn add_new_line(&mut self);
  fn add_spaces(&mut self, amount: u16);
  fn add_fragment(&mut self, fragment: &Fragment);
  fn add_styled<T: Into<Rc<str>>>(&mut self, style: Style, text: T);
  fn add_plain<T: Into<Rc<str>>>(&mut self, text: T);
}

impl BuildInput for Vec<Command> {
  fn add_new_line(&mut self) {
    self.push(Command::NewLine);
  }

  fn add_spaces(&mut self, amount: u16) {
    self.push(Command::Space(amount))
  }

  fn add_fragment(&mut self, fragment: &Fragment) {
    self.push(Command::SetStyle(fragment.style));
    self.push(Command::Text(fragment.text.clone()));
    self.push(Command::ResetStyle(fragment.style));
  }

  fn add_styled<T>(&mut self, style: Style, text: T)
  where
    T: Into<Rc<str>>,
  {
    self.push(Command::SetStyle(style));
    self.add_plain(text);
    self.push(Command::ResetStyle(style));
  }

  fn add_plain<T>(&mut self, text: T)
  where
    T: Into<Rc<str>>,
  {
    self.push(Command::Text(text.into()));
  }
}
