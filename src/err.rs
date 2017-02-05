use std::str::Utf8Error;
use std::num::ParseIntError;
use std::fmt;
use std::error;

#[derive(Debug)]
pub enum BencodeError {
    Utf8(Utf8Error),
    IntError(ParseIntError),
    DictionaryKeyNotString,
    UnexpectedCharacter(usize),
    UnexpectedEndOfInput,
}

impl fmt::Display for BencodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BencodeError::Utf8(ref err) => write!(f, "Utf8 error: {}", err),
            BencodeError::IntError(ref err) => write!(f, "Int error: {}", err),
            BencodeError::DictionaryKeyNotString => write!(f, "Dictionary key was not a string"),
            BencodeError::UnexpectedCharacter(ref position) => write!(f, "Unexpected character: position {}", position),
            BencodeError::UnexpectedEndOfInput => write!(f, "Unexpected end of input"),
        }
    }
}

impl error::Error for BencodeError {
    fn description(&self) -> &str {
        match *self {
            BencodeError::Utf8(ref err) => err.description(),
            BencodeError::IntError(ref err) => err.description(),
            BencodeError::DictionaryKeyNotString => "Dictionary key was not a string",
            BencodeError::UnexpectedCharacter(_) => "Unexpected character",
            BencodeError::UnexpectedEndOfInput => "Unexpected end of input",
        }
    }
}

impl From<Utf8Error> for BencodeError {
    fn from(err: Utf8Error) -> BencodeError {
        BencodeError::Utf8(err)
    }
}

impl From<ParseIntError> for BencodeError {
    fn from(err: ParseIntError) -> BencodeError {
        BencodeError::IntError(err)
    }
}

