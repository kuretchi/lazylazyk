use crate::encode::{ChurchNat, ScottList};
use crate::expr::{Expr, Thunk};

use derivative::Derivative;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::io::{self, prelude::*};
use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum MagicExpr {
    NatDecoder(NatDecoder),
    Input(Input),
}

impl MagicExpr {
    pub fn input(reader: Box<dyn Read>) -> Self {
        MagicExpr::Input(Input::new(reader))
    }

    pub fn reduce(self) -> io::Result<Expr> {
        match self {
            MagicExpr::Input(input) => input.evaluate()?.reduce(),
            _ => Ok(Expr::Magic(self)),
        }
    }

    pub fn reduce_apply(self, rhs: Thunk) -> io::Result<Expr> {
        match self {
            MagicExpr::NatDecoder(lhs) => lhs.reduce_apply(rhs),
            MagicExpr::Input(input) => Expr::Apply(input.evaluate()?.freeze(), rhs).reduce(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum NatDecoder {
    Nat(usize),
    Succ,
}

impl NatDecoder {
    pub fn reduce_apply(self, rhs: Thunk) -> io::Result<Expr> {
        match self {
            lhs @ NatDecoder::Succ => match rhs.thaw()? {
                Expr::Magic(MagicExpr::NatDecoder(NatDecoder::Nat(n))) => {
                    Ok(Expr::Magic(MagicExpr::NatDecoder(NatDecoder::Nat(n + 1))))
                }
                rhs => Ok(Expr::Apply(
                    Expr::Magic(MagicExpr::NatDecoder(lhs)).freeze(),
                    rhs.freeze(),
                )),
            },
            lhs => Ok(Expr::Apply(
                Expr::Magic(MagicExpr::NatDecoder(lhs)).freeze(),
                rhs,
            )),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum InputItem {
    Byte(u8),
    Eof,
}

#[derive(Clone, Debug)]
pub struct Input {
    index: usize,
    reader: Rc<RefCell<Reader>>,
    item: Rc<Cell<Option<InputItem>>>,
}

impl Input {
    fn new(reader: Box<dyn Read>) -> Self {
        Input {
            index: 0,
            reader: Rc::new(RefCell::new(Reader::new(reader))),
            item: Rc::new(Cell::new(None)),
        }
    }

    fn next(&self) -> Self {
        Input {
            index: self.index + 1,
            reader: Rc::clone(&self.reader),
            item: Rc::new(Cell::new(None)),
        }
    }

    fn evaluate(self) -> io::Result<Expr> {
        let n = match self.get()? {
            InputItem::Byte(byte) => byte as usize,
            InputItem::Eof => 256,
        };
        let car = ChurchNat::from(n);
        let cdr = Expr::Magic(MagicExpr::Input(self.next()));
        Ok(ScottList::cons(car.into(), cdr).into())
    }

    fn get(&self) -> io::Result<InputItem> {
        match self.item.get() {
            Some(item) => Ok(item),
            None => {
                let item = self.reader.borrow_mut().get(self.index)?;
                self.item.set(Some(item));
                Ok(item)
            }
        }
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
struct Reader {
    #[derivative(Debug = "ignore")]
    reader: Box<dyn Read>,
    cache: HashMap<usize, u8>,
    reached_eof: bool,
    next_index: usize,
}

impl Reader {
    fn new(reader: Box<dyn Read>) -> Self {
        Reader {
            reader,
            cache: HashMap::new(),
            reached_eof: false,
            next_index: 0,
        }
    }

    fn get(&mut self, index: usize) -> io::Result<InputItem> {
        if index < self.next_index {
            Ok(InputItem::Byte(self.cache.remove(&index).unwrap()))
        } else if self.reached_eof {
            Ok(InputItem::Eof)
        } else {
            let mut buf = [0];
            for i in self.next_index..=index {
                self.next_index += 1;
                match self.reader.read(&mut buf)? {
                    0 => {
                        self.reached_eof = true;
                        return Ok(InputItem::Eof);
                    }
                    1 => {
                        let byte = buf[0];
                        if i == index {
                            return Ok(InputItem::Byte(byte));
                        }
                        self.cache.insert(i, byte);
                    }
                    _ => unreachable!(),
                }
            }
            unreachable!()
        }
    }
}
