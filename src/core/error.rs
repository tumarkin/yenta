use ansi_term::Colour::Red;
use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub struct YentaWrappedError<T> {
    original_error: T,
    msg: String,
}

impl<T: Error> Display for YentaWrappedError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}: {}\n", Red.paint("error"), self.msg)?;
        std::fmt::Display::fmt(&self.original_error, f)
    }
}

impl<T: Error> Error for YentaWrappedError<T> {}

pub fn wrap_error<T: Error>(e: T, msg: String) -> YentaWrappedError<T> {
    YentaWrappedError {
        original_error: e,
        msg,
    }
}
