
use core::fmt;
use std::ops::{self, Div};
use std::str::FromStr;
use core::f64;

use malachite::{rational::Rational, Integer, Float, Natural};
use malachite::base::num::basic::traits::{One, Zero};

use crate::error::Error;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Num {
    Int(Integer),
    //Rat(Rational),
    Fp (f64),
}

impl From<f64> for Num {
    fn from(value: f64) -> Self {
        Num::Fp(value)
    }
}

impl Num {
    pub fn zero() -> Num {
        Num::Int(Integer::ZERO)
    }

    pub fn pi() -> Num {
        f64::consts::PI.into()
    }

    pub fn is_float(&self) -> bool {
        match &self {
            Self::Fp(_) => true,
            _ => false,
        }
    }

    pub fn is_integer(&self) -> bool {
        match &self {
            Self::Int(_) => true,
            _ => false,
        }
    }

    /// Converts the internal representation into a rational number.
    pub fn float(self) -> Num {
        match self {
            Self::Int(n) => Self::Fp(f64::from_str(&n.to_string()).unwrap_or_default()),
            Self::Fp(n) => Self::Fp(n),
        }
    }

    /// Converts the internal representation into an integer.
    pub fn integer(self) -> Num {
        match self {
            Self::Int(n) => Self::Int(n),
            Self::Fp(n) => {
                Natural::from_bits(n as i128)
            },
        }
    }
}

impl ops::Div for Num {
    type Output = Num;
    fn div(self, rhs: Self) -> Self::Output {
        use Num::*;
        match (self, rhs) {
            (Rat(l), Rat(r)) => Rat(l.div(r)),
            (l, r) => l.rational().div(r.rational()),
        }
    }
}

impl ops::Mul for Num {
    type Output = Num;
    fn mul(self, rhs: Self) -> Self::Output {
        use Num::*;
        match (self, rhs) {
            (Rat(l), Rat(r)) => Rat(l.mul(r)),
            (l, r) => l.rational().mul(r.rational()),
        }
    }
}

impl ops::Sub for Num {
    type Output = Num;
    fn sub(self, rhs: Self) -> Self::Output {
        use Num::*;
        match (self, rhs) {
            (Rat(l), Rat(r)) => Rat(l.sub(r)),
            (l, r) => l.rational().sub(r.rational()),
        }
    }
}

impl ops::Add for Num {
    type Output = Num;
    fn add(self, rhs: Self) -> Self::Output {
        use Num::*;
        match (self, rhs) {
            (Rat(l), Rat(r)) => Rat(l.add(r)),
            (l, r) => l.rational().add(r.rational()),
        }
    }
}

impl fmt::Display for Num {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Num::Rat(i) => write!(f, "{i}"),
            Num::Int(i) => write!(f, "{i}"),
        }
    }
}

impl FromStr for Num {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match Rational::from_str(s) {
            Ok(r) => Ok(Num::Rat(r)),
            Err(_) => Err(Error::reason("number: unable to convert string to number")),
        }
    }
}

