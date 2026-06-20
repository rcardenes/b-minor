use std::io;
use crate::types::PrimType;

trait Backend<W>
    where W: std::io::Write,
{
    type Reg: Copy;

    fn type_size(&self, dtype: PrimType) -> usize;

    fn func_preamble(&self, output: &mut W, func_name: &str) -> Result<(), io::Error>;
    fn func_postamble(&self, output: &mut W) -> Result<(), io::Error>;

    // Load/Store

    // Arith
    fn add(&mut self, output: &mut W, r1: Self::Reg, r2: Self::Reg) -> Result<Option<Self::Reg>, io::Error>;
    fn sub(&mut self, output: &mut W, r1: Self::Reg, r2: Self::Reg) -> Result<Option<Self::Reg>, io::Error>;
    fn mul(&mut self, output: &mut W, r1: Self::Reg, r2: Self::Reg) -> Result<Option<Self::Reg>, io::Error>;
    fn div(&mut self, output: &mut W, r1: Self::Reg, r2: Self::Reg) -> Result<Option<Self::Reg>, io::Error>;

    // Provided
    fn label(&self, output: &mut W, label_num: usize) -> Result<(), io::Error> {
        writeln!(output, ".L{label_num}")
    }

    fn file_preamble(&self, output: &mut W, file_name: &str) -> Result<(), io::Error> {
        writeln!(output, "\t.file\t\"{}\"", file_name)?;
        writeln!(output, "\t.text")?;
        writeln!(output, "\t.ident\t\"B-minor (2026-06-18 0.1)")
    }
}

struct CodeGenerator {
}

impl CodeGenerator {
//    fn start(
}
