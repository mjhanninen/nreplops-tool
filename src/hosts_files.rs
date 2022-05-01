use std::{
    collections::HashMap,
    ffi::OsString,
    fs,
    io::{self, Read},
    path::PathBuf,
};

use serde::Deserialize;

use crate::{host_options::{HostKey, HostOptions, HostOptionsTable}, conn_expr::ConnectionExpr};

// XXX(soija) Use crate::errors instead of raw io::Error

pub fn load_default_hosts_files() -> Result<HostOptionsTable, io::Error> {
    let mut hosts = HostOptionsTable::default();
    // XXX(soija) ConfigFiles does not implement DoubledEndedIterator
    #[allow(clippy::needless_collect)]
    let ps = matching_config_files("nreplops-hosts.toml")
        .unwrap()
        .collect::<Vec<_>>();
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

#[derive(Debug, Deserialize)]
pub struct HostOptionsDe {
    name: Option<String>,
    #[serde(with = "serde_with::rust::display_fromstr")]
    connection: ConnectionExpr,
    confirm: Option<bool>,
}

// XXX(soija) HostOptions is independent of HostOptionsDe
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

fn matching_config_files<S>(file_name: S) -> Result<ConfigFiles, io::Error>
where
    S: Into<OsString>,
{
    let dir = PathBuf::from(".").canonicalize()?;
    Ok(ConfigFiles {
        file_name: file_name.into(),
        dir,
        state: ConfigFilesState::User,
    })
}

impl Iterator for ConfigFiles {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        use ConfigFilesState::*;
        loop {
            match self.state {
                User => {
                    self.state = Ancestors;
                    if let Some(mut item) = config_dir() {
                        item.push(&self.file_name);
                        if item.is_file() {
                            return Some(item);
                        }
                    }
                }
                Ancestors => {
                    let mut item = self.dir.clone();
                    if !self.dir.pop() {
                        self.state = Done
                    }
                    item.push(&self.file_name);
                    if item.is_file() {
                        return Some(item);
                    }
                }
                Done => return None,
            }
        }
    }
}

#[derive(Debug)]
struct ConfigFiles {
    file_name: OsString,
    dir: PathBuf,
    state: ConfigFilesState,
}

#[derive(Debug)]
enum ConfigFilesState {
    User,
    Ancestors,
    Done,
}

fn config_dir() -> Option<PathBuf> {
    let mut dir = dirs::config_dir()?;
    dir.push("nr");
    Some(dir)
}
