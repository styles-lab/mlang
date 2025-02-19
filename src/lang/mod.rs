//! Compile and code generation tools for mlang.

pub mod analyzer;
pub mod ir;
pub mod parser;
pub mod rustgen;

mod ext {

    use parserc::{ParseContext, Result};

    use super::{analyzer::semantic_analyze, parser::parse, rustgen::CodeGen};

    /// Compile `mlang` source code and generate rust source code.
    ///
    /// This function will output any errors encountered during compilation directly to the terminal
    pub fn compile<S: AsRef<str>>(source: S, codegen: CodeGen) -> Result<()> {
        let mut ctx = ParseContext::from(source.as_ref());

        let mut stats = match parse(&mut ctx) {
            Ok(stats) => stats,
            Err(err) => {
                return Err(err);
            }
        };

        if !semantic_analyze(&mut stats) {
            return Err(parserc::ControlFlow::Fatal);
        }

        match codegen.gen(stats) {
            Err(err) => {
                eprintln!("codegen: {}", err);
                return Err(parserc::ControlFlow::Fatal);
            }
            _ => {}
        }

        Ok(())
    }
}

pub use ext::*;
