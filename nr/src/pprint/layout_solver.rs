// pprint/layout_solver.rs
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

use super::{
  fragments::{Fragment, FragmentText},
  printer::Command,
  style::Style,
};

#[derive(Clone, Debug)]
pub enum Chunk<'a> {
  /// Marks horizontal
  PushAnchor,
  //
  // XXX(soija) FIXME: Instead using a stacked anchors use indexed ones and
  //            remove the pop.
  //
  /// Removes the latest anchor
  PopAnchor,
  /// Inserts space, except at the start of the line
  SoftSpace,
  /// Unconditional line break
  HardBreak,
  /// Unbreakable strip of fragments
  Text(Box<[Fragment<'a>]>),
}

#[derive(Default)]
pub struct TextBuilder<'a> {
  fragments: Vec<Fragment<'a>>,
}

impl<'a> TextBuilder<'a> {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn add<T: Into<FragmentText<'a>>>(
    mut self,
    text: T,
    style: Style,
  ) -> Self {
    self.fragments.push(Fragment::new(style, text));
    self
  }

  pub fn apply<F>(self, mut func: F) -> Self
  where
    F: FnOnce(Self) -> Self,
  {
    func(self)
  }

  pub fn build(mut self) -> Chunk<'a> {
    self.fragments.shrink_to_fit();
    Chunk::Text(self.fragments.into_boxed_slice())
  }
}

pub fn solve<'a>(chunks: &[Chunk<'a>], printer_input: &mut Vec<Command<'a>>) {
  use super::printer::BuildInput;
  use Chunk as C;

  let mut col = 0_u16;
  let mut anchors = vec![0_u16];

  for c in chunks {
    match c {
      C::Text(fragments) => {
        for f in fragments.iter() {
          printer_input.add_fragment(f);
          col += f.width() as u16;
        }
      }
      C::SoftSpace => {
        printer_input.add_spaces(1);
        col += 1;
      }
      C::HardBreak => {
        printer_input.add_new_line();
        col = *anchors.last().unwrap();
        printer_input.add_spaces(col);
      }
      C::PushAnchor => {
        anchors.push(col);
      }
      C::PopAnchor => {
        anchors.pop();
      }
    }
  }
}
