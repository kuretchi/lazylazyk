use crate::expr::Expr::{self, I, K, S};
use crate::magic::{MagicExpr, NatDecoder};

use std::io;
use std::iter;

#[derive(Clone)]
pub struct ChurchNat(Expr);

impl From<usize> for ChurchNat {
    fn from(n: usize) -> Self {
        iter::successors(Some(ChurchNat::zero()), |n| Some(n.clone().succ()))
            .nth(n)
            .unwrap()
    }
}

impl Into<Expr> for ChurchNat {
    fn into(self) -> Expr {
        self.0
    }
}

impl ChurchNat {
    fn zero() -> Self {
        ChurchNat(K.apply(I))
    }

    fn succ(self) -> Self {
        ChurchNat(S.apply(S.apply(K.apply(S)).apply(K)).apply(self.0))
    }

    pub fn decode(n: Expr) -> io::Result<Option<usize>> {
        match n
            .apply(Expr::Magic(MagicExpr::NatDecoder(NatDecoder::Succ)))
            .apply(Expr::Magic(MagicExpr::NatDecoder(NatDecoder::Nat(0))))
            .reduce()?
        {
            Expr::Magic(MagicExpr::NatDecoder(NatDecoder::Nat(n))) => Ok(Some(n)),
            _ => Ok(None),
        }
    }
}

#[derive(Clone)]
pub struct ScottList(Expr);

impl From<Expr> for ScottList {
    fn from(expr: Expr) -> Self {
        ScottList(expr)
    }
}

impl Into<Expr> for ScottList {
    fn into(self) -> Expr {
        self.0
    }
}

impl ScottList {
    pub fn cons(car: Expr, cdr: Expr) -> Self {
        ScottList(S.apply(S.apply(I).apply(K.apply(car))).apply(K.apply(cdr)))
    }

    pub fn uncons(self) -> (Expr, Expr) {
        let this = self.0.freeze();
        let car = Expr::Apply(this.clone(), K.freeze());
        let cdr = Expr::Apply(this, K.apply(I).freeze());
        (car, cdr)
    }
}
