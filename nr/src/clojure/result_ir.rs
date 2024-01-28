// result_ir.rs
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

use super::lex::Lexeme;

#[derive(Debug)]
pub enum Value<'a> {
  Nil,
  Number {
    literal: &'a str,
  },
  String {
    literal: &'a str,
  },
  Boolean {
    value: bool,
  },
  SymbolicValue {
    literal: &'a str,
  },
  Symbol {
    namespace: Option<&'a str>,
    name: &'a str,
  },
  Keyword {
    namespace: KeywordNamespace<'a>,
    name: &'a str,
  },
  TaggedLiteral {
    namespace: Option<&'a str>,
    name: &'a str,
    value: Box<Value<'a>>,
  },
  List {
    values: Box<[Value<'a>]>,
  },

  Vector {
    values: Box<[Value<'a>]>,
  },

  Set {
    values: Box<[Value<'a>]>,
  },

  Map {
    entries: Box<[MapEntry<'a>]>,
  },
}

#[derive(Debug)]
pub enum KeywordNamespace<'a> {
  None,
  Alias(&'a str),
  Namespace(&'a str),
}

#[derive(Debug)]
pub struct MapEntry<'a> {
  pub key: Value<'a>,
  pub value: Value<'a>,
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
}

pub fn build<'a>(lexemes: &[Lexeme<'a>]) -> Result<Value<'a>, BuildError> {
  use Lexeme::*;
  let mut b = Builder::new();
  for lexeme in lexemes {
    let is_complete = match lexeme {
      Whitespace { .. } | SymbolicValuePrefix { .. } => false, // ignore
      StartList { .. } => b.start(ContainerType::List)?,
      EndList { .. } => b.end(ContainerType::List)?,
      StartSet { .. } => b.start(ContainerType::Set)?,
      EndSet { .. } => b.end(ContainerType::Set)?,
      StartVector { .. } => b.start(ContainerType::Vector)?,
      EndVector { .. } => b.end(ContainerType::Vector)?,
      StartMap { .. } => b.start(ContainerType::Map)?,
      EndMap { .. } => b.end(ContainerType::Map)?,
      Nil { .. } => b.add_nil()?,
      Boolean { value, .. } => b.add_boolean(*value)?,
      Numeric { source, .. } => b.add_number(source)?,
      String { source, .. } => b.add_string(source)?,
      SymbolicValue { source, .. } => b.add_symbolic_value(source)?,
      // NB: The tagged literal builder expects that the tag is passed on as a
      //     symbol.  This way we don't need to add a separate "tag" value that
      //     would stick out of the enum like a sore thumb.
      Symbol {
        namespace, name, ..
      }
      | Tag {
        namespace, name, ..
      } => b.add_symbol(name, *namespace)?,
      Keyword {
        alias,
        namespace,
        name,
        ..
      } => b.add_keyword(name, *namespace, *alias)?,
      TaggedLiteral { .. } => b.start(ContainerType::TaggedLiteral)?,
      unhandled => todo!("Missing rule for:\n{:#?}", unhandled),
    };
    if is_complete {
      while b.finalize_current_value()? {}
    }
  }
  b.build()
}

struct Builder<'a> {
  stack: Vec<ContainerBuilder<'a>>,
}

impl<'a> Builder<'a> {
  fn new() -> Self {
    Self {
      stack: vec![ContainerBuilder::new(ContainerType::TopLevel)],
    }
  }

  fn add_to_topmost(&mut self, value: Value<'a>) -> Result<bool, BuildError> {
    self.stack.last_mut().unwrap().add(value)
  }

  fn build(mut self) -> Result<Value<'a>, BuildError> {
    // We can unwrap safely ⇐ last one is a top-level builder and we have
    // asserted the type of the other ones when popping them out of the
    // builder stack.
    let b = self.stack.pop().unwrap();
    if b.container_type() == ContainerType::TopLevel {
      b.build()
    } else {
      Err(BuildError::RunawayCollection)
    }
  }

  fn start(
    &mut self,
    container_type: ContainerType,
  ) -> Result<bool, BuildError> {
    self.stack.push(ContainerBuilder::new(container_type));
    Ok(false)
  }

  fn end(&mut self, container_type: ContainerType) -> Result<bool, BuildError> {
    if self.stack.last().unwrap().container_type() == container_type {
      Ok(true)
    } else {
      Err(BuildError::InconsistentCollections)
    }
  }

  fn finalize_current_value(&mut self) -> Result<bool, BuildError> {
    let b = self.stack.pop().unwrap();
    self.stack.last_mut().unwrap().add(b.build()?)
  }

  fn add_nil(&mut self) -> Result<bool, BuildError> {
    self.add_to_topmost(Value::Nil)
  }

  fn add_boolean(&mut self, value: bool) -> Result<bool, BuildError> {
    self.add_to_topmost(Value::Boolean { value })
  }

  fn add_number(&mut self, literal: &'a str) -> Result<bool, BuildError> {
    self.add_to_topmost(Value::Number { literal })
  }

  fn add_string(&mut self, literal: &'a str) -> Result<bool, BuildError> {
    self.add_to_topmost(Value::String { literal })
  }

  fn add_symbolic_value(
    &mut self,
    literal: &'a str,
  ) -> Result<bool, BuildError> {
    self.add_to_topmost(Value::SymbolicValue { literal })
  }

  fn add_symbol(
    &mut self,
    name: &'a str,
    namespace: Option<&'a str>,
  ) -> Result<bool, BuildError> {
    self.add_to_topmost(Value::Symbol { name, namespace })
  }

  fn add_keyword(
    &mut self,
    name: &'a str,
    namespace: Option<&'a str>,
    alias: bool,
  ) -> Result<bool, BuildError> {
    use KeywordNamespace as N;
    let value = Value::Keyword {
      name,
      namespace: if let Some(n) = namespace {
        if alias {
          N::Alias(n)
        } else {
          N::Namespace(n)
        }
      } else {
        N::None
      },
    };
    self.add_to_topmost(value)
  }
}

trait BuildContainer<'a> {
  fn container_type(&self) -> ContainerType;

  /// Adds a contained value to the value being built.  Returns `true` if value
  /// being built is complete and should be popped out.
  fn add(&mut self, value: Value<'a>) -> Result<bool, BuildError>;

  fn build(self) -> Result<Value<'a>, BuildError>;
}

/// The types of value containers we recognize at the level of syntax.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ContainerType {
  List,
  Vector,
  Set,
  Map,
  /// A tagged literal
  ///
  /// The tag is guaranteed to be a symbol whereas the value can be any value.
  TaggedLiteral,
  /// The top-level of the program (or the result).  Currently limited to hold
  /// no more nor less than a single value.
  TopLevel,
}

// The reason we have this intermediate builder instead of using `Box<dyn
// BuildContainer<'a> + a>` is that moving the contained values out of turned
// out to be surprisingly clunky. The extra boilerplate introduced by this
// is annoying but at least it is concentrated in one place.
enum ContainerBuilder<'a> {
  TopLevel(TopLevelBuilder<'a>),
  Seq(SeqBuilder<'a>),
  Map(MapBuilder<'a>),
  TaggedLiteral(TaggedLiteralBuilder<'a>),
}

impl<'a> ContainerBuilder<'a> {
  fn new(container_type: ContainerType) -> Self {
    use ContainerType as T;
    match container_type {
      T::TopLevel => Self::TopLevel(TopLevelBuilder::new()),
      T::List => Self::Seq(SeqBuilder::new(SeqType::List)),
      T::Vector => Self::Seq(SeqBuilder::new(SeqType::Vector)),
      T::Set => Self::Seq(SeqBuilder::new(SeqType::Set)),
      T::Map => Self::Map(MapBuilder::new()),
      T::TaggedLiteral => Self::TaggedLiteral(Default::default()),
    }
  }
}

impl<'a> BuildContainer<'a> for ContainerBuilder<'a> {
  fn container_type(&self) -> ContainerType {
    match self {
      ContainerBuilder::Seq(b) => b.container_type(),
      ContainerBuilder::Map(b) => b.container_type(),
      ContainerBuilder::TopLevel(b) => b.container_type(),
      ContainerBuilder::TaggedLiteral(b) => b.container_type(),
    }
  }

  fn add(&mut self, value: Value<'a>) -> Result<bool, BuildError> {
    match self {
      ContainerBuilder::Seq(b) => b.add(value),
      ContainerBuilder::Map(b) => b.add(value),
      ContainerBuilder::TopLevel(b) => b.add(value),
      ContainerBuilder::TaggedLiteral(b) => b.add(value),
    }
  }

  fn build(self) -> Result<Value<'a>, BuildError> {
    match self {
      ContainerBuilder::Seq(b) => b.build(),
      ContainerBuilder::Map(b) => b.build(),
      ContainerBuilder::TopLevel(b) => b.build(),
      ContainerBuilder::TaggedLiteral(b) => b.build(),
    }
  }
}

struct SeqBuilder<'a> {
  seq_type: SeqType,
  values: Vec<Value<'a>>,
}

impl<'a> SeqBuilder<'a> {
  fn new(seq_type: SeqType) -> Self {
    SeqBuilder {
      seq_type,
      values: Vec::new(),
    }
  }
}

impl<'a> BuildContainer<'a> for SeqBuilder<'a> {
  fn container_type(&self) -> ContainerType {
    match self.seq_type {
      SeqType::List => ContainerType::List,
      SeqType::Vector => ContainerType::Vector,
      SeqType::Set => ContainerType::Set,
    }
  }

  fn add(&mut self, value: Value<'a>) -> Result<bool, BuildError> {
    self.values.push(value);
    Ok(false)
  }

  fn build(mut self) -> Result<Value<'a>, BuildError> {
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
enum TaggedLiteralBuilder<'a> {
  #[default]
  Empty,
  WithTag {
    namespace: Option<&'a str>,
    name: &'a str,
  },
  WithTagAndValue {
    namespace: Option<&'a str>,
    name: &'a str,
    value: Value<'a>,
  },
  Invalid,
}

impl<'a> BuildContainer<'a> for TaggedLiteralBuilder<'a> {
  fn container_type(&self) -> ContainerType {
    ContainerType::TaggedLiteral
  }

  fn add(&mut self, value: Value<'a>) -> Result<bool, BuildError> {
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
      _ => Err(BuildError::ExpectedValueForTaggedLiteral),
    }
  }

  fn build(self) -> Result<Value<'a>, BuildError> {
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
      Err(BuildError::IncompleteTaggedLiteral)
    }
  }
}

#[derive(Default)]
struct MapBuilder<'a> {
  key: Option<Value<'a>>,
  entries: Vec<MapEntry<'a>>,
}

impl<'a> MapBuilder<'a> {
  fn new() -> Self {
    Default::default()
  }
}

impl<'a> BuildContainer<'a> for MapBuilder<'a> {
  fn container_type(&self) -> ContainerType {
    ContainerType::Map
  }

  fn add(&mut self, value: Value<'a>) -> Result<bool, BuildError> {
    if let Some(key) = self.key.take() {
      self.entries.push(MapEntry { key, value });
    } else {
      self.key = Some(value);
    }
    Ok(false)
  }

  fn build(mut self) -> Result<Value<'a>, BuildError> {
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
struct TopLevelBuilder<'a> {
  value: Option<Value<'a>>,
}

impl<'a> TopLevelBuilder<'a> {
  fn new() -> Self {
    Default::default()
  }
}

impl<'a> BuildContainer<'a> for TopLevelBuilder<'a> {
  fn container_type(&self) -> ContainerType {
    ContainerType::TopLevel
  }

  fn add(&mut self, value: Value<'a>) -> Result<bool, BuildError> {
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

  fn build(self) -> Result<Value<'a>, BuildError> {
    self.value.ok_or(BuildError::TooFewTopLevelValues)
  }
}
