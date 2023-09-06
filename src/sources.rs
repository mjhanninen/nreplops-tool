// sources.rs
// Copyright 2022 Matti Hänninen
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
    borrow::Cow,
    collections::HashMap,
    fs,
    io::{self, Read},
    rc::Rc,
};

use crate::{cli, error::Error};

#[derive(Debug)]
pub struct Source {
    pub content: String,
    pub file: Option<String>,
}

pub fn load_sources(
    source_args: &[cli::SourceArg],
    template_args: &[cli::TemplateArg],
) -> Result<Vec<Source>, Error> {
    let context = Context::from(template_args);
    let mut result = Vec::new();
    for source_arg in source_args.iter() {
        let (file, raw_content) = load_content(source_arg)?;
        let content = render_source(&context, raw_content.as_ref());
        result.push(Source { content, file });
    }
    Ok(result)
}

fn load_content(
    source_arg: &cli::SourceArg,
) -> Result<(Option<String>, Cow<'_, str>), Error> {
    use cli::SourceArg::*;
    match source_arg {
        Pipe => {
            let stdin = io::stdin();
            let mut handle = stdin.lock();
            let mut buffer = String::new();
            handle
                .read_to_string(&mut buffer)
                .map_err(|_| Error::CannotReadStdIn)?;
            Ok((None, Cow::Owned(buffer)))
        }
        Expr(e) => Ok((None, Cow::Borrowed(e.as_str()))),
        File(f) => {
            let mut file = fs::File::open(f).map_err(|_| {
                Error::CannotReadFile(f.to_string_lossy().to_string())
            })?;
            let mut buffer = String::new();
            file.read_to_string(&mut buffer).map_err(|_| {
                Error::CannotReadFile(f.to_string_lossy().to_string())
            })?;
            Ok((Some(f.to_string_lossy().to_string()), Cow::Owned(buffer)))
        }
    }
}

// XXX(soija) This needs work
// This rendering has the following limitations:
// - does not catch '#nr[...]' exprs without value arg (nREPL catches this though)
// - is not easy to extend supporting '#nr[<var> <default>]'
// - let alone '#nr[<var-1> ... <var-n> <default>]'
// - does not captures values from environment variables (e.g. NR_VAR_1)
fn render_source(context: &Context, source: &str) -> String {
    let after_shebang = if source.starts_with("#!") {
        match source.split_once('\n') {
            Some((_, remaining)) => remaining,
            None => "",
        }
    } else {
        source
    }
    .trim();
    if let Some(ref regex) = context.regex {
        let mut fragments = Vec::<Rc<str>>::new();
        let mut remaining = after_shebang;
        while let Some(captures) = regex.captures(remaining) {
            let full_match = captures.get(0).unwrap();
            let (upto, after) = remaining.split_at(full_match.end());
            let (before, _) = upto.split_at(full_match.start());
            fragments.push(before.to_string().into());
            let value = context
                .table
                .get(captures.get(1).unwrap().as_str())
                .unwrap();
            fragments.push(value.clone());
            remaining = after;
        }
        fragments.push(remaining.to_string().into());
        fragments.join("")
    } else {
        after_shebang.into()
    }
}

#[derive(Debug)]
struct Context {
    table: HashMap<Rc<str>, Rc<str>>,
    regex: Option<regex::Regex>,
}

impl From<&[cli::TemplateArg]> for Context {
    fn from(template_args: &[cli::TemplateArg]) -> Self {
        let table = template_args.iter().fold(HashMap::new(), |mut m, a| {
            if let Some(ref n) = a.name {
                m.insert(n.clone(), a.value.clone());
            }
            if let Some(i) = a.pos {
                m.insert((i + 1).to_string().into(), a.value.clone());
            }
            m
        });
        let regex = if table.is_empty() {
            None
        } else {
            let keys = table
                .keys()
                .map(|s| regex::escape(s))
                .collect::<Vec<String>>();
            let key_union = keys.join("|");
            let pat = format!(r#"#nr\s*\[\s*({})\s*\]"#, key_union,);
            // XXX(soija) Once https://github.com/rust-lang/rust/issues/79524 lands, use
            // intersperse.
            /*
            let pat = format!(
                r#"#nr\s*\[\s*({})\s*\]"#,
                table
                    .keys()
                    .map(|k| regex::escape(k))
                    .intersperse("|".to_string())
                    .collect::<String>(),
            );
            */
            Some(regex::Regex::new(&pat).unwrap())
        };
        Self { table, regex }
    }
}
