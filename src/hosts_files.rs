// host_files.rs
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
  collections::HashMap,
  env, fs,
  io::{self, Read},
  path::{Path, PathBuf},
};

use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};

use crate::{
  conn_expr::ConnectionExpr,
  host_options::{HostKey, HostOptions, HostOptionsTable},
};

// XXX(soija) Use crate::errors instead of raw io::Error

pub fn load_default_hosts_files() -> Result<HostOptionsTable, io::Error> {
  let mut hosts = HostOptionsTable::default();
  let ps = matching_config_files("nreplops-hosts.toml")?;
  for p in ps.into_iter().rev() {
    let mut f = fs::File::open(p)?;
    let mut s = String::new();
    let _ = f.read_to_string(&mut s)?;
    let new_hosts: Hosts = toml::from_str(&s).unwrap();
    hosts.extend(new_hosts.into_iter().map(|(k, v)| (k, v.into())));
  }
  Ok(hosts)
}

pub type Hosts = HashMap<HostKey, HostOptionsDe>;

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct HostOptionsDe {
  name: Option<String>,
  #[serde_as(as = "DisplayFromStr")]
  connection: ConnectionExpr,
  confirm: Option<bool>,
}

// `HostOptions` is independent of `HostOptionsDe` and, hence, we prefer
// implementing `Into`.
#[allow(clippy::from_over_into)]
impl Into<HostOptions> for HostOptionsDe {
  fn into(self) -> HostOptions {
    HostOptions {
      name: self.name,
      conn_expr: self.connection,
      ask_confirmation: self.confirm,
    }
  }
}

fn matching_config_files<P>(file_name: P) -> Result<Vec<PathBuf>, io::Error>
where
  P: AsRef<Path>,
{
  let mut found = Vec::new();

  let mut current = PathBuf::from(".").canonicalize()?;
  loop {
    let mut p = current.clone();
    p.push(file_name.as_ref());
    if p.is_file() {
      found.push(p);
    }
    if !current.pop() {
      break;
    }
  }

  if let Some(dir) = env::var_os("HOME") {
    let mut p = PathBuf::from(dir);
    if p.is_absolute() {
      p.push("Library");
      p.push("Application Support");
      p.push("nreplops");
      p.push(file_name.as_ref());
      if p.is_file() {
        found.push(p);
      }
    }
  }

  if let Some(dir) = env::var_os("XDG_CONFIG_HOME") {
    let mut p = PathBuf::from(dir);
    if p.is_absolute() {
      p.push("nreplops");
      p.push(file_name.as_ref());
      if p.is_file() {
        found.push(p);
      }
    }
  }

  if let Some(dir) = env::var_os("HOME") {
    let mut p = PathBuf::from(dir);
    if p.is_absolute() {
      p.push(".nreplops");
      p.push(file_name.as_ref());
      if p.is_file() {
        found.push(p);
      }
    }
  }

  Ok(found)
}
