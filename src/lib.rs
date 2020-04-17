mod encode;
mod expr;
mod magic;
mod parse;

use encode::{ChurchNat, ScottList};
use expr::Expr;

use std::io::{self, prelude::*};

#[derive(Clone, Debug)]
pub struct Program(Expr);

pub use parse::{parse, ParseError};

pub fn run<W>(reader: Box<dyn Read>, mut writer: W, prog: Program) -> io::Result<Option<i32>>
where
    W: Write,
{
    let mut expr = Expr::Apply(prog.0.freeze(), Expr::input(reader).freeze());
    loop {
        let (car, cdr) = ScottList::from(expr).uncons();
        expr = cdr;
        match ChurchNat::decode(car)? {
            Some(n) => {
                if n >= 256 {
                    let exit_code = (n - 256) as i32;
                    break Ok(Some(exit_code));
                }
                writer.write_all(&[n as u8])?;
                writer.flush()?;
            }
            None => break Ok(None),
        }
    }
}
