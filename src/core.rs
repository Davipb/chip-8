use std::fmt::{self, Display, Debug, Formatter};
use std::io;
use std::ops::{Add, Sub, BitOr, BitAnd, BitXor, AddAssign, SubAssign, Index, IndexMut};
use std::num::Wrapping;
use std::borrow::Cow;
use ctrlc;

pub type ResultChip8<T> = Result<T, Error>;
pub type VoidResultChip8 = ResultChip8<()>;

#[derive(Debug, Clone)]
pub struct Error {
    message: String,
    cause: Option<Box<Error>>,
}

impl Error {
    pub fn new(message: String) -> Error {
        Error {
            message,
            cause: None,
        }
    }

    pub fn new_str(message: &str) -> Error {
        Error {
            message: message.to_owned(),
            cause: None,
        }
    }

    pub fn chain(self, message: String) -> Error {
        Error {
            message,
            cause: Some(Box::new(self)),
        }
    }
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{}", self.message)?;
        if let Some(cause) = &self.cause {
            fmt.write_str(" Caused by: ")?;
            write!(fmt, "{}", cause)?;
        }
        Ok(())
    }
}

impl From<io::Error> for Error {
    fn from(other: io::Error) -> Error {
        Error::new(other.to_string())
    }
}

impl From<fmt::Error> for Error {
    fn from(_: fmt::Error) -> Error {
        Error::new_str("Formatting error")
    }
}

impl From<ctrlc::Error> for Error {
    fn from(other: ctrlc::Error) -> Error {
        Error::new(other.to_string())
    }
}

impl From<Error> for Cow<'_, Error> {
    fn from(x: Error) -> Self {
        Cow::Owned(x)
    }
}

impl std::error::Error for Error {}


#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Hash)]
pub struct Word(Wrapping<u8>);

impl Word {
    pub const ZERO: Word = Word(Wrapping(0));

    pub fn new(value: impl Into<u8>) -> Word {
        Word(Wrapping(value.into()))
    }
}

impl Display for Word {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:02X}", self.0)
    }
}

impl Debug for Word {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl From<Word> for u8 {
    fn from(x: Word) -> Self {
        Self::from(x.0.0)
    }
}

impl From<Word> for u16 {
    fn from(x: Word) -> Self {
        Self::from(x.0.0)
    }
}

impl From<u8> for Word {
    fn from(x: u8) -> Self {
        Self::new(x)
    }
}

impl From<Word> for Cow<'_, Word> {
    fn from(x: Word) -> Self {
        Cow::Owned(x)
    }
}

impl<T> Add<T> for Word where T : Into<Word> {
    type Output = Word;
    fn add(self, rhs: T) -> Word {
        let rhs: Word = rhs.into();
        (self.0 + rhs.0).0.into()
    }
}

impl<T> AddAssign<T> for Word where T : Into<Word> {
    fn add_assign(&mut self, rhs: T) {
        let rhs: Word = rhs.into();
        self.0 += rhs.0;
    }
}

impl<T> Sub<T> for Word where T : Into<Word> {
    type Output = Word;
    fn sub(self, rhs: T) -> Word {
        let rhs: Word = rhs.into();
        (self.0 - rhs.0).0.into()
    }
}

impl<T> SubAssign<T> for Word where T : Into<Word> {
    fn sub_assign(&mut self, rhs: T) {
        let rhs: Word = rhs.into();
        self.0 -= rhs.0;
    }
}

impl<T> BitAnd<T> for Word where T : Into<Word> {
    type Output = Word;
    fn bitand(self, rhs: T) -> Word {
        let rhs: Word = rhs.into();
        (self.0 & rhs.0).0.into()
    }
}

impl<T> BitOr<T> for Word where T : Into<Word> {
    type Output = Word;
    fn bitor(self, rhs: T) -> Word {
        let rhs: Word = rhs.into();
        (self.0 | rhs.0).0.into()
    }
}

impl<T> BitXor<T> for Word where T : Into<Word> {
    type Output = Word;
    fn bitxor(self, rhs: T) -> Word {
        let rhs: Word = rhs.into();
        (self.0 ^ rhs.0).0.into()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Hash)]
pub struct Address(Wrapping<u16>);

impl Address {
    pub const ZERO: Address = Address(Wrapping(0));

    pub fn new(value: impl Into<u16>) -> Address {
        Address(Wrapping(value.into()))
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:04X}", self.0)
    }
}

impl Debug for Address {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl From<Address> for usize {
    fn from(x: Address) -> Self {
        Self::from(x.0.0)
    }
}

impl From<usize> for Address {
    fn from(x: usize) -> Self {
        Address::new(x as u16)
    }
}

impl From<i32> for Address {
    fn from(x: i32) -> Self {
        Address::new(x as u16)
    }
}

impl From<Address> for u16 {
    fn from(x: Address) -> Self {
        x.0.0
    }
}

impl From<u16> for Address {
    fn from(x: u16) -> Self {
        Address::new(x)
    }
}

impl From<u8> for Address {
    fn from(x: u8) -> Self {
        Address::new(x)
    }
}

impl From<Word> for Address {
    fn from(x: Word) -> Self {
        Address::new(x)
    }
}

impl From<Address> for Cow<'_, Address> {
    fn from(x: Address) -> Self {
        Cow::Owned(x)
    }
}

impl<T> Add<T> for Address where T : Into<Address> {
    type Output = Address;
    fn add(self, rhs: T) -> Address {
        let rhs: Address = rhs.into();
        (self.0 + rhs.0).0.into()
    }
}

impl<T> AddAssign<T> for Address where T : Into<Address> {
    fn add_assign(&mut self, rhs: T) {
        let rhs: Address = rhs.into();
        self.0 += rhs.0;
    }
}

impl<T> Sub<T> for Address where T : Into<Address> {
    type Output = Address;
    fn sub(self, rhs: T) -> Address {
        let rhs: Address = rhs.into();
        (self.0 - rhs.0).0.into()
    }
}

impl<T> SubAssign<T> for Address where T : Into<Address> {
    fn sub_assign(&mut self, rhs: T) {
        let rhs: Address = rhs.into();
        self.0 -= rhs.0;
    }
}
