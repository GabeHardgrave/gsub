pub struct Presenter {
    verbose: bool,
}

impl Presenter {
    pub fn new(verbose: bool) -> Self {
        Self { verbose: verbose }
    }

    pub fn wax<S>(&self, msg: S)
        where S: std::fmt::Display
    {
        println!("{}", msg)
    }

    pub fn wax_verbose<F, S>(&self, msg: F)
        where S: std::fmt::Display,
              F: FnOnce() -> S
    {
        if self.verbose {
            println!("{}", msg())
        }
    }
}