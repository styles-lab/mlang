//! `mlang` code generator for rust language.

pub mod mapping;

mod opcode;
pub use opcode::*;
mod serde;
pub use serde::*;

mod ext {
    use std::{
        io::{Error, ErrorKind, Result},
        path::{Path, PathBuf},
    };

    use proc_macro2::TokenStream;
    use quote::quote;

    use crate::lang::{ir::Stat, rustgen::gen_opcode_mod};

    use super::gen_serde_mod;

    fn write_and_fmt_rs<C: AsRef<[u8]>, P: AsRef<Path>>(path: P, content: C) -> Result<()> {
        println!("codegen({:?}):", path.as_ref());

        std::fs::write(path.as_ref(), content).map_err(|err| {
            Error::new(
                ErrorKind::Other,
                format!("write file {:?} error: {}", path.as_ref(), err),
            )
        })?;

        println!("    write file ... ok");

        std::process::Command::new("rustfmt")
            .arg(path.as_ref())
            .output()
            .map_err(|err| {
                Error::new(
                    ErrorKind::Other,
                    format!("run rustfmt for {:?} error: {}", path.as_ref(), err),
                )
            })?;

        println!("    run rustfmt ... ok");

        Ok(())
    }

    /// A builder to config and generate rust source code.
    pub struct CodeGen {
        with_serde: bool,
        target: PathBuf,
    }

    impl Default for CodeGen {
        fn default() -> Self {
            Self {
                with_serde: true,
                target: Path::new("./").to_path_buf(),
            }
        }
    }

    impl CodeGen {
        /// Reset `serde` module generation flag, the default value is true.
        pub fn with_serde(mut self, on: bool) -> Self {
            self.with_serde = on;
            self
        }

        /// Reset the target path of the code generation, the default value is `current directory`.
        pub fn target(mut self, path: impl AsRef<Path>) -> Self {
            self.target = path.as_ref().to_path_buf();
            self
        }

        /// invoke real rust code generation processing.
        pub fn codegen(self, stats: impl AsRef<[Stat]>) -> Result<()> {
            if !self.target.exists() {
                std::fs::create_dir_all(&self.target)?;
            }

            let mut mods = vec![("opcode", gen_opcode_mod(stats.as_ref()))];

            if self.with_serde {
                mods.push(("serde", gen_serde_mod(stats.as_ref(), "super::opcode::")));
            }

            let mut impls = vec![];

            for (name, codes) in mods {
                let target_file = self.target.join(format!("{}.rs", name));

                write_and_fmt_rs(target_file, codes.to_string())?;

                let ident = name.parse::<TokenStream>().unwrap();

                impls.push(quote! {
                    pub mod #ident;
                });
            }

            let codes = quote! {
                //! This module is automatically generated by the ml compiler, do not modify it manually.

                #(#impls)*
            };

            let target_file = self.target.join("mod.rs");

            write_and_fmt_rs(target_file, codes.to_string())?;

            Ok(())
        }
    }
}

pub use ext::*;
