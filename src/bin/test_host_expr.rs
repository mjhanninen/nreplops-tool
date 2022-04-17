use std::env;

use nreplops_tool::host_expression::parser::*;
use pest::Parser as _;

fn main() {
    for arg in env::args().skip(1) {
        println!("INPUT: {}", arg);
        let result = Parser::parse(Rule::host_expr, arg.as_str());
        println!("RESULT:\n{:#?}", result);
    }
}
