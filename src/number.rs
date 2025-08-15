
use core::fmt;
use core::ops::{Div, Mul, Sub, Add};
use core::str::FromStr;
use core::f64;

use malachite::Integer;
use malachite::base::num::basic::traits::{One, Zero};

use crate::error::Error;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Num {
    Int(Integer),
    Fp (f64),
}

impl From<f64> for Num {
    fn from(value: f64) -> Self {
        Num::Fp(value)
    }
}

impl Num {
    #[allow(dead_code)]
    pub fn zero() -> Num {
        Num::Int(Integer::ZERO)
    }

    pub fn pi() -> Num {
        f64::consts::PI.into()
    }

    pub fn e() -> Num {
        f64::consts::E.into()
    }

    #[allow(dead_code)]
    pub fn is_float(&self) -> bool {
        match &self {
            Num::Fp(_) => true,
            _ => false,
        }
    }

    #[allow(dead_code)]
    pub fn is_integer(&self) -> bool {
        match &self {
            Num::Int(_) => true,
            _ => false,
        }
    }

    /// Converts the internal representation into a float number.
    pub fn float(self) -> Num {
        match self {
            Num::Int(n) => Num::Fp(f64::from_str(&n.to_string()).unwrap_or_default()),
            Num::Fp(n) => Num::Fp(n),
        }
    }

    /// Converts the internal representation into an integer.
    #[allow(dead_code)]
    pub fn integer(self) -> Num {
        match self {
            Num::Int(n) => Num::Int(n),
            Num::Fp(n) => {
                Num::Int((n as i128).into())
            },
        }
    }

    pub fn ln(self) -> Num {
        match self {
            Num::Fp(f) => Num::Fp(f.ln()),
            _ => self.float().ln(),
        }
    }

    pub fn log2(self) -> Num {
        match self {
            Num::Fp(f) => Num::Fp(f.log2()),
            _ => self.float().log2(),
        }
    }

    pub fn log10(self) -> Num {
        match self {
            Num::Fp(f) => Num::Fp(f.log10()),
            _ => self.float().log10(),
        }
    }

    pub fn log(self, other: Num) -> Num {
        match (self, other) {
            (Num::Fp(fs), Num::Fp(fo)) => Num::Fp(fs.log(fo)),
            (s, o) => s.float().log(o.float()),
        }
    }

    pub fn pow(self, n: &Num) -> Num {
        match n {
            Num::Int(int) => {
                match self {
                    Num::Int(n) => {
                        let mut count = int.clone();
                        let mut prod = n.clone();
                        while count.gt(&Integer::ONE) {
                            prod *= n.clone();
                            count -= Integer::ONE;
                        }
                        Num::Int(prod)
                    },
                    Num::Fp(n) => {
                        let mut count = int.clone();
                        let mut prod = n;
                        while count.gt(&f64::ONE) {
                            prod *= n;
                            count -= Integer::ONE;
                        }
                        Num::Fp(prod)
                    },
                }
            },
            Num::Fp(f) => {
                let flt = match self.float() {
                    Num::Fp(f) => f,
                    _ => unreachable!(),
                };
                Num::Fp(flt.powf(*f))
            },
        }
    }

    pub(crate) fn sqrt(self) -> Num {
        match self {
            Num::Fp(f) => Num::Fp(f.sqrt()),
            _ => self.float().sqrt(),
        }
    }

    pub(crate) fn exp(self) -> Num {
        Num::e().pow(&self)
    }

    pub(crate) fn abs(self) -> Num {
        match self {
            Num::Fp(f) => Num::Fp(f.abs()),
            Num::Int(n) => Num::Int(n.unsigned_abs_ref().into())
        }
    }
}

impl Div for Num {
    type Output = Num;
    fn div(self, rhs: Self) -> Self::Output {
        use Num::*;
        match (self, rhs) {
            (Int(l), Int(r)) => Int(l.div(r)),
            (l, r) => l.float().div(r.float()),
        }
    }
}

impl Mul for Num {
    type Output = Num;
    fn mul(self, rhs: Self) -> Self::Output {
        use Num::*;
        match (self, rhs) {
            (Int(l), Int(r)) => Int(l.mul(r)),
            (l, r) => l.float().mul(r.float()),
        }
    }
}

impl Sub for Num {
    type Output = Num;
    fn sub(self, rhs: Self) -> Self::Output {
        use Num::*;
        match (self, rhs) {
            (Int(l), Int(r)) => Int(l.sub(r)),
            (l, r) => l.float().sub(r.float()),
        }
    }
}

impl Add for Num {
    type Output = Num;
    fn add(self, rhs: Self) -> Self::Output {
        use Num::*;
        match (self, rhs) {
            (Int(l), Int(r)) => Int(l.add(r)),
            (l, r) => l.float().add(r.float()),
        }
    }
}

impl fmt::Display for Num {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Num::Fp(n) => write!(f, "{n}"),
            Num::Int(n) => write!(f, "{n}"),
        }
    }
}

impl FromStr for Num {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let int_result = Integer::from_str(s);
        if let Ok(int) = int_result {
            return Ok(Num::Int(int));
        }

        let fp_result = f64::from_str(s);
        if let Ok(fp) = fp_result {
            return Ok(Num::Fp(fp));
        }

        Error::reason("number: unable to convert string to number").into()
    }
}

