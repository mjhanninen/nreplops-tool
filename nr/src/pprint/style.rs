// style.rs
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

use anstyle::AnsiColor;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
  Whitespace,
  CollectionDelimiter,
  SymbolDecoration,
  SymbolNamespace,
  SymbolName,
  KeywordDecoration,
  KeywordNamespace,
  KeywordName,
  VarQuoteDecoration,
  VarQuoteNamespace,
  VarQuoteName,
  StringDecoration,
  StringValue,
  NumberValue,
  BooleanValue,
  NilValue,
  SymbolicValueDecoration,
  SymbolicValue,
  TaggedLiteralDecoration,
  TaggedLiteralNamespace,
  TaggedLiteralName,
}

impl Style {
  pub fn to_ansi_color(self) -> AnsiColor {
    use AnsiColor as A;
    use Style as S;
    match self {
      S::Whitespace => A::BrightBlack,
      S::CollectionDelimiter => A::White,
      S::SymbolDecoration => A::BrightBlack,
      S::SymbolNamespace => A::BrightBlack,
      S::SymbolName => A::White,
      S::KeywordDecoration => A::BrightBlack,
      S::KeywordNamespace => A::BrightBlack,
      S::KeywordName => A::BrightBlue,
      S::VarQuoteDecoration => A::BrightBlack,
      S::VarQuoteNamespace => A::BrightBlack,
      S::VarQuoteName => A::White,
      S::StringDecoration => A::BrightBlack,
      S::StringValue => A::BrightGreen,
      S::NumberValue => A::White,
      S::BooleanValue => A::White,
      S::NilValue => A::Red,
      S::SymbolicValueDecoration => A::BrightBlack,
      S::SymbolicValue => A::Red,
      S::TaggedLiteralDecoration => A::BrightBlack,
      S::TaggedLiteralNamespace => A::BrightBlack,
      S::TaggedLiteralName => A::Yellow,
    }
  }
}
