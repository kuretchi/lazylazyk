use crate::magic::MagicExpr;

use derivative::Derivative;
use std::cell::Cell;
use std::fmt::{self, Debug, Formatter};
use std::io::{self, Read};
use std::rc::Rc;

#[derive(Clone, Debug, Derivative)]
#[derivative(Default)]
pub enum Expr {
    S,
    K,
    #[derivative(Default)]
    I,
    Iota,
    S1(Thunk),
    S2(Thunk, Thunk),
    K1(Thunk),
    Apply(Thunk, Thunk),
    Magic(MagicExpr),
}

impl Expr {
    pub fn apply(self, rhs: Self) -> Self {
        Expr::Apply(self.freeze(), rhs.freeze())
    }

    pub fn input(reader: Box<dyn Read>) -> Self {
        Expr::Magic(MagicExpr::input(reader))
    }

    pub fn reduce(self) -> io::Result<Expr> {
        let mut this = self;
        loop {
            match this {
                Expr::Apply(lhs, rhs) => match lhs.thaw()? {
                    Expr::S => break Ok(Expr::S1(rhs)),
                    Expr::K => break Ok(Expr::K1(rhs)),
                    Expr::I => break rhs.thaw(),
                    Expr::Iota => {
                        this = Expr::Apply(
                            Expr::Apply(rhs, Expr::S.freeze()).freeze(),
                            Expr::K.freeze(),
                        )
                    }
                    Expr::S1(arg0) => break Ok(Expr::S2(arg0, rhs)),
                    Expr::S2(arg0, arg1) => {
                        this = Expr::Apply(
                            Expr::Apply(arg0, rhs.clone()).freeze(),
                            Expr::Apply(arg1, rhs).freeze(),
                        )
                    }
                    Expr::K1(arg0) => break arg0.thaw(),
                    lhs @ Expr::Apply(_, _) => break Ok(Expr::Apply(lhs.freeze(), rhs)),
                    Expr::Magic(expr) => break expr.reduce_apply(rhs),
                },
                Expr::Magic(expr) => break expr.reduce(),
                _ => break Ok(this),
            }
        }
    }

    pub fn freeze(self) -> Thunk {
        Thunk(Rc::new(Cell::new(self)))
    }
}

#[derive(Clone)]
pub struct Thunk(Rc<Cell<Expr>>);

impl Debug for Thunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let expr = self.0.take();
        write!(f, "{:?}", expr)?;
        self.0.set(expr);
        Ok(())
    }
}

impl Thunk {
    pub fn thaw(&self) -> io::Result<Expr> {
        let expr = self.0.take().reduce()?;
        self.0.set(expr.clone());
        Ok(expr)
    }
}
