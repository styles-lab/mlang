//! vglang metadata parser implementation.

mod error;
pub use error::*;
mod field;
mod link;
mod lit;
mod node;
mod prop;
mod stat;
mod utils;

use parserc::{IntoParser, ParseContext, Parser, ParserExt, Result};

use crate::lang::ir::Stat;

/// Parse input source code.
pub fn parse(input: &mut ParseContext<'_>) -> Result<Vec<Stat>, ParseError> {
    let mut opcodes = vec![];

    while let Some(opcode) = Stat::into_parser().ok().parse(input)? {
        opcodes.push(opcode);
    }

    Ok(opcodes)
}
