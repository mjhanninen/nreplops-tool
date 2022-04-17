use pest_derive::Parser;

#[allow(missing_debug_implementations)]
#[derive(Parser)]
#[grammar = "host_expression/grammar.pest"]
pub struct Parser;
