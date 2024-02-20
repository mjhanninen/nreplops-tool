// clojure/result_ir.rs
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

//! Data structures and tools for our internal representation of Clojure result
//! values.

// XXX(soija) This whole thing is a hot mess.  Redesign it BUT NOT BEFORE trying
//            to use it for splitting and rewriting the forms that are sent to
//            the nREPL server.

use std::rc::Rc;

use super::lex::{Lexeme, Token};

#[derive(Debug)]
pub enum Value {
  Nil,
  Number {
    literal: Rc<str>,
  },
  String {
    literal: Rc<str>,
  },
  Boolean {
    value: bool,
  },
  SymbolicValue {
    literal: Rc<str>,
  },
  Symbol {
    namespace: Option<Rc<str>>,
    name: Rc<str>,
  },
  Keyword {
    namespace: Option<Rc<str>>,
    name: Rc<str>,
    alias: bool,
  },
  TaggedLiteral {
    namespace: Option<Rc<str>>,
    name: Rc<str>,
    value: Box<Value>,
  },
  VarQuoted {
    namespace: Option<Rc<str>>,
    name: Rc<str>,
  },
  List {
    values: Box<[Value]>,
  },

  Vector {
    values: Box<[Value]>,
  },

  Set {
    values: Box<[Value]>,
  },
  Map {
    entries: Box<[MapEntry]>,
  },
}

#[derive(Debug)]
pub struct MapEntry {
  pub key: Value,
  pub value: Value,
}

#[derive(Debug)]
pub enum BuildError {
  TooFewTopLevelValues,
  TooManyTopLevelValues,
  RunawayCollection,
  InconsistentCollections,
  IncompleteMapEntry,
  ExpectedTagForTaggedLiteral,
  ExpectedValueForTaggedLiteral,
  IncompleteTaggedLiteral,
  UnexpectedLiteralTag,
  ExpectedSymbolForLiteralTag,
  ExpectedSymbolForVarQuote,
}

pub fn build(lexemes: &[Lexeme]) -> Result<Value, BuildError> {
  use Token as T;

  let mut b = Builder::new();

  for l in lexemes {
    let mut composite_ready = match &l.token {
      T::Whitespace | T::SymbolicValuePrefix => false, // ignore
      T::Nil => b.add_to_composite(Value::Nil)?,
      T::Boolean { value } => {
        b.add_to_composite(Value::Boolean { value: *value })?
      }
      T::Numeric { .. } => b.add_to_composite(Value::Number {
        // XXX(soija) FIXME: Extract the value from the token, not from the
        //            lexeme.
        literal: l.source.as_ref().unwrap().str.clone(),
      })?,
      T::String { raw_value, .. } => b.add_to_composite(Value::String {
        literal: raw_value.clone().into(),
      })?,
      T::SymbolicValue { .. } => {
        b.add_to_composite(Value::SymbolicValue {
          // XXX(soija) FIXME: Extract the value from the token, not from the
          //            lexeme.
          literal: l.source.as_ref().unwrap().str.clone(),
        })?
      }
      // NB: The tagged literal builder expects that the tag is passed on as
      //     a symbol.  This way we don't need to add a separate `Value::Tag`
      //     value that would stick out of the enum like a sore thumb.
      T::Symbol { namespace, name } | T::Tag { namespace, name } => b
        .add_to_composite(Value::Symbol {
          name: name.clone(),
          namespace: namespace.clone(),
        })?,
      T::Keyword {
        alias,
        namespace,
        name,
      } => b.add_to_composite(Value::Keyword {
        name: name.clone(),
        namespace: namespace.clone(),
        alias: *alias,
      })?,
      T::StartList => b.start(CompositeType::List)?,
      T::EndList => b.end(CompositeType::List)?,
      T::StartSet => b.start(CompositeType::Set)?,
      T::EndSet => b.end(CompositeType::Set)?,
      T::StartVector => b.start(CompositeType::Vector)?,
      T::EndVector => b.end(CompositeType::Vector)?,
      T::StartMap => b.start(CompositeType::Map)?,
      T::EndMap => b.end(CompositeType::Map)?,
      T::TaggedLiteral { .. } => b.start(CompositeType::TaggedLiteral)?,
      T::VarQuote => b.start(CompositeType::VarQuoted)?,

      _ => todo!("Missing rule for: {l:#?}"),
    };

    while composite_ready {
      composite_ready = b.build_composite()?;
    }
  }

  b.build_top_level()
}

struct Builder {
  stack: Vec<CompositeBuilder>,
}

impl Builder {
  fn new() -> Self {
    Self {
      stack: vec![CompositeBuilder::new(CompositeType::TopLevel)],
    }
  }

  fn start(
    &mut self,
    composite_type: CompositeType,
  ) -> Result<bool, BuildError> {
    self.stack.push(CompositeBuilder::new(composite_type));
    Ok(false)
  }

  fn end(&mut self, composite_type: CompositeType) -> Result<bool, BuildError> {
    if self.stack.last().unwrap().composite_type() == composite_type {
      Ok(true)
    } else {
      Err(BuildError::InconsistentCollections)
    }
  }

  fn add_to_composite(&mut self, value: Value) -> Result<bool, BuildError> {
    self.stack.last_mut().unwrap().add(value)
  }

  fn build_composite(&mut self) -> Result<bool, BuildError> {
    let b = self.stack.pop().unwrap();
    self.add_to_composite(b.build()?)
  }

  fn build_top_level(mut self) -> Result<Value, BuildError> {
    // We can unwrap safely ⇐ last one is a top-level builder and we have
    // asserted the type of the other ones when popping them out of the
    // builder stack.
    let b = self.stack.pop().unwrap();
    if b.composite_type() == CompositeType::TopLevel {
      b.build()
    } else {
      Err(BuildError::RunawayCollection)
    }
  }
}

trait BuildComposite {
  fn composite_type(&self) -> CompositeType;

  /// Adds a contained value to the value being built.  Returns `true` if value
  /// being built is complete and should be popped out.
  fn add(&mut self, value: Value) -> Result<bool, BuildError>;

  fn build(self) -> Result<Value, BuildError>;
}

/// The types of comosite values we recognize at the level of syntax.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CompositeType {
  List,
  Vector,
  Set,
  Map,
  /// A tagged literal
  ///
  /// The tag is guaranteed to be a symbol whereas the value can be any value.
  TaggedLiteral,
  /// A varquoted symbol
  VarQuoted,
  /// The top-level of the program (or the result)
  ///
  /// Currently limited to hold no more nor less than a single value.
  TopLevel,
}

// The reason we have this intermediate builder instead of using `Box<dyn
// BuildContainer<'a> + a>` is that moving the contained values out of turned
// out to be surprisingly clunky. The extra boilerplate introduced by this
// is annoying but at least it is concentrated in one place.
//
// XXX(soija) Okay, this keeps growing and is getting unbearably clunky.
//
enum CompositeBuilder {
  TopLevel(TopLevelBuilder),
  Seq(SeqBuilder),
  Map(MapBuilder),
  TaggedLiteral(TaggedLiteralBuilder),
  VarQuoted(VarQuotedBuilder),
}

impl CompositeBuilder {
  fn new(composite_type: CompositeType) -> Self {
    use CompositeType as T;
    match composite_type {
      T::TopLevel => Self::TopLevel(TopLevelBuilder::new()),
      T::List => Self::Seq(SeqBuilder::new(SeqType::List)),
      T::Vector => Self::Seq(SeqBuilder::new(SeqType::Vector)),
      T::Set => Self::Seq(SeqBuilder::new(SeqType::Set)),
      T::Map => Self::Map(Default::default()),
      T::TaggedLiteral => Self::TaggedLiteral(Default::default()),
      T::VarQuoted => Self::VarQuoted(Default::default()),
    }
  }
}

impl BuildComposite for CompositeBuilder {
  fn composite_type(&self) -> CompositeType {
    match self {
      CompositeBuilder::Seq(b) => b.composite_type(),
      CompositeBuilder::Map(b) => b.composite_type(),
      CompositeBuilder::TopLevel(b) => b.composite_type(),
      CompositeBuilder::TaggedLiteral(b) => b.composite_type(),
      CompositeBuilder::VarQuoted(b) => b.composite_type(),
    }
  }

  fn add(&mut self, value: Value) -> Result<bool, BuildError> {
    match self {
      CompositeBuilder::Seq(b) => b.add(value),
      CompositeBuilder::Map(b) => b.add(value),
      CompositeBuilder::TopLevel(b) => b.add(value),
      CompositeBuilder::TaggedLiteral(b) => b.add(value),
      CompositeBuilder::VarQuoted(b) => b.add(value),
    }
  }

  fn build(self) -> Result<Value, BuildError> {
    match self {
      CompositeBuilder::Seq(b) => b.build(),
      CompositeBuilder::Map(b) => b.build(),
      CompositeBuilder::TopLevel(b) => b.build(),
      CompositeBuilder::TaggedLiteral(b) => b.build(),
      CompositeBuilder::VarQuoted(b) => b.build(),
    }
  }
}

struct SeqBuilder {
  seq_type: SeqType,
  values: Vec<Value>,
}

impl SeqBuilder {
  fn new(seq_type: SeqType) -> Self {
    SeqBuilder {
      seq_type,
      values: Vec::new(),
    }
  }
}

impl BuildComposite for SeqBuilder {
  fn composite_type(&self) -> CompositeType {
    match self.seq_type {
      SeqType::List => CompositeType::List,
      SeqType::Vector => CompositeType::Vector,
      SeqType::Set => CompositeType::Set,
    }
  }

  fn add(&mut self, value: Value) -> Result<bool, BuildError> {
    self.values.push(value);
    Ok(false)
  }

  fn build(mut self) -> Result<Value, BuildError> {
    self.values.shrink_to_fit();
    let values = self.values.into_boxed_slice();
    Ok(match self.seq_type {
      SeqType::List => Value::List { values },
      SeqType::Vector => Value::Vector { values },
      SeqType::Set => Value::Set { values },
    })
  }
}

enum SeqType {
  List,
  Vector,
  Set,
}

#[derive(Default)]
enum TaggedLiteralBuilder {
  #[default]
  Empty,
  WithTag {
    namespace: Option<Rc<str>>,
    name: Rc<str>,
  },
  WithTagAndValue {
    namespace: Option<Rc<str>>,
    name: Rc<str>,
    value: Value,
  },
  Invalid,
}

impl BuildComposite for TaggedLiteralBuilder {
  fn composite_type(&self) -> CompositeType {
    CompositeType::TaggedLiteral
  }

  fn add(&mut self, value: Value) -> Result<bool, BuildError> {
    use TaggedLiteralBuilder as B;
    match std::mem::replace(self, B::Invalid) {
      B::Empty => match value {
        Value::Symbol { namespace, name } => {
          *self = B::WithTag { namespace, name };
          Ok(false)
        }
        _ => Err(BuildError::ExpectedSymbolForLiteralTag),
      },
      B::WithTag { namespace, name } => {
        *self = B::WithTagAndValue {
          namespace,
          name,
          value,
        };
        Ok(true)
      }
      // XXX(soija) This is probably a wrong error here.  We should probably
      //            panic (unreachable) here.  FIXME: Write a test case and
      //            figure this out.
      _ => Err(BuildError::ExpectedValueForTaggedLiteral),
    }
  }

  fn build(self) -> Result<Value, BuildError> {
    if let TaggedLiteralBuilder::WithTagAndValue {
      namespace,
      name,
      value,
    } = self
    {
      Ok(Value::TaggedLiteral {
        namespace,
        name,
        value: value.into(),
      })
    } else {
      // XXX(soija) This is probably a wrong error here.  FIXME: Write a test
      //            case and figure this out.
      Err(BuildError::IncompleteTaggedLiteral)
    }
  }
}

#[derive(Default)]
enum VarQuotedBuilder {
  #[default]
  Empty,
  WithSymbol {
    namespace: Option<Rc<str>>,
    name: Rc<str>,
  },
  Invalid,
}

impl BuildComposite for VarQuotedBuilder {
  fn composite_type(&self) -> CompositeType {
    CompositeType::VarQuoted
  }

  fn add(&mut self, value: Value) -> Result<bool, BuildError> {
    use VarQuotedBuilder as B;
    match std::mem::replace(self, VarQuotedBuilder::Invalid) {
      B::Empty => match value {
        Value::Symbol { namespace, name } => {
          *self = VarQuotedBuilder::WithSymbol { namespace, name };
          Ok(true)
        }
        _ => Err(BuildError::ExpectedSymbolForVarQuote),
      },
      _ => unreachable!("should have triggered error or build"),
    }
  }

  fn build(self) -> Result<Value, BuildError> {
    if let VarQuotedBuilder::WithSymbol { namespace, name } = self {
      Ok(Value::VarQuoted { namespace, name })
    } else {
      // But what if we ran out of lexemes?
      unreachable!("should not have been triggered unless ready");
    }
  }
}

#[derive(Default)]
struct MapBuilder {
  key: Option<Value>,
  entries: Vec<MapEntry>,
}

impl BuildComposite for MapBuilder {
  fn composite_type(&self) -> CompositeType {
    CompositeType::Map
  }

  fn add(&mut self, value: Value) -> Result<bool, BuildError> {
    if let Some(key) = self.key.take() {
      self.entries.push(MapEntry { key, value });
    } else {
      self.key = Some(value);
    }
    Ok(false)
  }

  fn build(mut self) -> Result<Value, BuildError> {
    if self.key.is_none() {
      self.entries.shrink_to_fit();
      Ok(Value::Map {
        entries: self.entries.into_boxed_slice(),
      })
    } else {
      Err(BuildError::IncompleteMapEntry)
    }
  }
}

#[derive(Default)]
struct TopLevelBuilder {
  value: Option<Value>,
}

impl TopLevelBuilder {
  fn new() -> Self {
    Default::default()
  }
}

impl BuildComposite for TopLevelBuilder {
  fn composite_type(&self) -> CompositeType {
    CompositeType::TopLevel
  }

  fn add(&mut self, value: Value) -> Result<bool, BuildError> {
    if self.value.is_none() {
      self.value = Some(value);
      // XXX(soija) Actually, it would be consistent to return `true` in here
      //            but the way the main building loop is implemented requires
      //            that we return here false.  FIXME: do it right.
      Ok(false)
    } else {
      Err(BuildError::TooManyTopLevelValues)
    }
  }

  fn build(self) -> Result<Value, BuildError> {
    self.value.ok_or(BuildError::TooFewTopLevelValues)
  }
}
