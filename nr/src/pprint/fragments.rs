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

use std::rc::Rc;

use super::style::Style;

#[derive(Clone, Debug)]
pub struct Fragment {
  pub style: Style,
  pub text: Rc<str>,
}

impl Fragment {
  pub fn new<T>(style: Style, text: T) -> Self
  where
    T: Into<Rc<str>>,
  {
    Self {
      style,
      text: text.into(),
    }
  }

  pub fn width(&self) -> usize {
    self.text.chars().count()
  }
}
