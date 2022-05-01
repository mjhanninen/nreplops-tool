use std::collections::HashMap;

use crate::conn_expr::ConnectionExpr;

pub type HostKey = String;

pub type HostOptionsTable = HashMap<HostKey, HostOptions>;

#[derive(Debug)]
pub struct HostOptions {
    pub name: Option<String>,
    pub conn_expr: ConnectionExpr,
    pub ask_confirmation: Option<bool>,
}
