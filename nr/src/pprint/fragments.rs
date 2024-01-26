// pprint/fragments.rs
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

use std::borrow::Borrow;

use super::style::Style;

#[derive(Clone, Debug)]
pub struct Fragment<'a> {
  pub style: Style,
  pub text: FragmentText<'a>,
}

impl<'a> Fragment<'a> {
  pub fn new<T>(style: Style, text: T) -> Self
  where
    T: Into<FragmentText<'a>>,
  {
    Self {
      style,
      text: text.into(),
    }
  }

  pub fn width(&self) -> usize {
    self.text.as_str().chars().count()
  }
}

#[derive(Clone, Debug)]
pub enum FragmentText<'a> {
  Borrowed(&'a str),
  Owned(Box<str>),
}

impl<'a> FragmentText<'a> {
  pub fn as_str(&'a self) -> &'a str {
    self.borrow()
  }
}

impl<'a> From<&'a str> for FragmentText<'a> {
  fn from(s: &'a str) -> Self {
    FragmentText::Borrowed(s)
  }
}

impl<'a> From<String> for FragmentText<'a> {
  fn from(s: String) -> Self {
    FragmentText::Owned(s.into_boxed_str())
  }
}

impl<'a> Borrow<str> for FragmentText<'a> {
  fn borrow(&self) -> &str {
    match self {
      FragmentText::Borrowed(s) => s,
      FragmentText::Owned(s) => s.borrow(),
    }
  }
}
