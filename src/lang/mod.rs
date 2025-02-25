//! Compile and code generation tools for mlang.

pub mod analyzer;
pub mod ir;
pub mod parser;
pub mod rustgen;

mod ext {

    use parserc::{ParseContext, Result};

    use super::{
        analyzer::semantic_analyze,
        parser::{ParseError, parse},
        rustgen::CodeGen,
    };

    /// Compile `mlang` source code and generate rust source code.
    ///
    /// This function will output any errors encountered during compilation directly to the terminal
    pub fn compile<S: AsRef<str>>(source: S, codegen: CodeGen) -> Result<(), ParseError> {
        let mut ctx = ParseContext::from(source.as_ref());

        let mut stats = match parse(&mut ctx) {
            Ok(stats) => stats,
            Err(err) => {
                return Err(err);
            }
        };

        if !semantic_analyze(&mut stats) {
            return Err(parserc::ControlFlow::Fatal(ParseError::Semantic));
        }

        match codegen.codegen(stats) {
            Err(err) => {
                eprintln!("codegen: {}", err);
                return Err(parserc::ControlFlow::Fatal(ParseError::Io(err.to_string())));
            }
            _ => {}
        }

        Ok(())
    }
}

pub use ext::*;
