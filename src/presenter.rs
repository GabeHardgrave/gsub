use std::{fmt, io};

pub enum Msg<M> {
    Important(M),
    Verbose(M),
}

pub trait ToMsg: fmt::Display + Sized  {
    fn important(self) -> Msg<String> { Msg::Important(self.to_string()) }
    fn verbose(self) -> Msg<String> { Msg::Verbose(self.to_string()) }
}

impl ToMsg for io::Error {}
impl ToMsg for String {}

pub struct Presenter {
    verbose: bool,
}

impl Presenter {
    pub fn new(verbose: bool) -> Self {
        Self { verbose: verbose }
    }

    pub fn wax<M>(&self, msg: Msg<M>) where M: fmt::Display {
        match (msg, self.verbose) {
            (Msg::Important(m), _) => println!("{}", m),
            (Msg::Verbose(m), true) => eprintln!("{}", m),
            _ => {}
        }
    }
}