use std::cmp::*;
use std::fmt;
use std::fmt::Formatter;
use std::ops::*;

#[derive(Debug, Clone)]
pub enum Object {
    Num(f64),
    Str(String),
    Bool(bool),
    Nil,
    ArithmeticError,
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Object::Num(n) => write!(f, "{n}"),
            Object::Str(n) => write!(f, "{n}"),
            Object::Bool(n) => {
                if *n {
                    write!(f, "true")
                } else {
                    write!(f, "false")
                }
            }
            Object::Nil => write!(f, "nil"),
            Object::ArithmeticError => panic!("Should not be trying to print this object"),
        }
    }
}

impl Sub for Object {
    type Output = Object;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Object::Num(left), Object::Num(right)) => Object::Num(left - right),
            _ => Object::ArithmeticError,
        }
    }
}

impl Div for Object {
    type Output = Object;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Object::Num(left), Object::Num(right)) => {
                if right == 0 as f64 {
                    Object::ArithmeticError
                } else {
                    Object::Num(left / right)
                }
            }
            _ => Object::ArithmeticError,
        }
    }
}

impl Mul for Object {
    type Output = Object;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Object::Num(left), Object::Num(right)) => Object::Num(left * right),
            _ => Object::ArithmeticError,
        }
    }
}

impl Add for Object {
    type Output = Object;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Object::Num(left), Object::Num(right)) => Object::Num(left + right),
            (Object::Str(left), Object::Str(right)) => Object::Str(format!("{}{}", left, right)),
            _ => Object::ArithmeticError,
        }
    }
}

impl PartialEq<Self> for Object {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Object::Num(left), Object::Num(right)) => left.eq(right),
            (Object::Str(left), Object::Str(right)) => left.eq(right),
            (Object::Bool(left), Object::Bool(right)) => left.eq(right),
            (Object::Nil, Object::Nil) => true,
            _ => false,
        }
    }
}

impl PartialOrd<Self> for Object {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Object::Num(left), Object::Num(right)) => left.partial_cmp(right),
            (Object::Str(left), Object::Str(right)) => left.partial_cmp(right),
            (Object::Bool(left), Object::Bool(right)) => left.partial_cmp(right),
            (Object::Nil, Object::Nil) => Some(Ordering::Equal),
            _ => None,
        }
    }
}
