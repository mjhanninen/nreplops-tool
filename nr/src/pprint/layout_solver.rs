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
  /// Set the given `anchor` at the current horizontal position offset by the
  /// `offset` amount.
  SetAnchor { anchor: Anchor },
  SetRelativeAnchor {
    anchor: Anchor,
    base: Anchor,
    offset: i16,
  },
  /// Inserts space, except at the start of the line
  SoftSpace,
  /// Break the line unconditionally and start the new line at the `anchor`
  /// position.
  HardBreak { anchor: Anchor },
  /// Jumps horizontally to the `anchor` position.
  JumpTo { anchor: Anchor },
  /// Unbreakable strip of fragments
  Text(Box<[Fragment<'a>]>),
}

#[derive(Debug)]
pub struct Chunks<'a> {
  anchor_count: usize,
  chunks: Box<[Chunk<'a>]>,
}

#[derive(Debug, Default)]
pub struct ChunkBuilder<'a> {
  anchor_count: usize,
  chunks: Vec<Chunk<'a>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Anchor(usize);

impl<'a> ChunkBuilder<'a> {
  pub fn new() -> Self {
    Default::default()
  }

  fn alloc_anchor(&mut self) -> Anchor {
    let ix = self.anchor_count;
    self.anchor_count += 1;
    Anchor(ix)
  }

  pub fn set_anchor(&mut self) -> Anchor {
    let anchor = self.alloc_anchor();
    self.chunks.push(Chunk::SetAnchor { anchor });
    anchor
  }

  pub fn set_relative_anchor(&mut self, base: Anchor, offset: i16) -> Anchor {
    let anchor = self.alloc_anchor();
    self.chunks.push(Chunk::SetRelativeAnchor {
      anchor,
      base,
      offset,
    });
    anchor
  }

  pub fn break_hard(&mut self, anchor: Anchor) {
    self.chunks.push(Chunk::HardBreak { anchor });
  }

  pub fn add_soft_space(&mut self) {
    self.chunks.push(Chunk::SoftSpace);
  }

  pub fn jump_to(&mut self, anchor: Anchor) {
    self.chunks.push(Chunk::JumpTo { anchor });
  }

  pub fn add_text<T>(&mut self, text: T)
  where
    T: Into<Box<[Fragment<'a>]>>,
  {
    self.chunks.push(Chunk::Text(text.into()))
  }

  pub fn build(self) -> Chunks<'a> {
    let ChunkBuilder {
      anchor_count,
      mut chunks,
    } = self;
    chunks.shrink_to_fit();
    Chunks {
      anchor_count,
      chunks: chunks.into_boxed_slice(),
    }
  }
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

  pub fn apply<F>(self, func: F) -> Self
  where
    F: FnOnce(Self) -> Self,
  {
    func(self)
  }

  pub fn build(mut self) -> Box<[Fragment<'a>]> {
    self.fragments.shrink_to_fit();
    self.fragments.into_boxed_slice()
  }
}

// Unfortunately we cannot implement the dual `From<...>` rule.
#[allow(clippy::from_over_into)]
impl<'a> Into<Box<[Fragment<'a>]>> for TextBuilder<'a> {
  fn into(self) -> Box<[Fragment<'a>]> {
    self.build()
  }
}

pub fn solve<'a>(chunks: &Chunks<'a>, printer_input: &mut Vec<Command<'a>>) {
  use super::printer::BuildInput;
  use Chunk as C;

  let Chunks {
    anchor_count,
    chunks,
  } = chunks;

  let mut column = 0_u16;
  let mut anchors = vec![0_u16; *anchor_count];

  for c in chunks.iter() {
    match c {
      C::Text(fragments) => {
        for f in fragments.iter() {
          printer_input.add_fragment(f);
          column += f.width() as u16;
        }
      }
      C::SoftSpace => {
        printer_input.add_spaces(1);
        column += 1;
      }
      C::HardBreak { anchor } => {
        printer_input.add_new_line();
        column = anchors[anchor.0];
        printer_input.add_spaces(column);
      }
      C::JumpTo { anchor } => {
        let target = anchors[anchor.0];
        if column < target {
          let jump = target - column;
          printer_input.add_spaces(jump);
          column = target;
        }
      }
      C::SetAnchor { anchor } => {
        anchors[anchor.0] = column;
      }
      C::SetRelativeAnchor {
        anchor,
        base,
        offset,
      } => {
        anchors[anchor.0] = (i16::try_from(anchors[base.0]).unwrap() + offset)
          .try_into()
          .unwrap();
      }
    }
  }
}
