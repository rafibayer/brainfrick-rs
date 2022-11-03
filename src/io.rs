use core::panic;
use std::{cell::RefCell, collections::VecDeque, io::Read, rc::Rc};

/// Generic IO trait
pub trait InputOutput {
    /// Get a single byte of input
    fn getch(&self) -> u8;
    /// Output a single byte
    fn print(&self, byte: u8);
}

/// InputOutput implementor for stdin/stdout
pub struct StdIO {}

impl InputOutput for StdIO {
    #[inline]
    fn getch(&self) -> u8 {
        std::io::stdin().bytes().next().unwrap().unwrap()
    }

    #[inline]
    fn print(&self, byte: u8) {
        print!("{}", byte as char)
    }
}

/// Test InputOutput implementor
pub struct TestIO {
    input: RefCell<VecDeque<u8>>,
    output: RefCell<String>,
}

#[cfg(test)]
impl TestIO {
    pub fn new(input: &str) -> Self {
        TestIO {
            // note: bytes instead of char, trusting test input here
            input: RefCell::new(input.to_string().bytes().collect()),
            output: RefCell::new(String::new()),
        }
    }

    pub fn output(&self) -> String {
        self.output.borrow().clone()
    }
}

impl InputOutput for Rc<TestIO> {
    fn getch(&self) -> u8 {
        self.input.borrow_mut().pop_front().unwrap()
    }

    fn print(&self, byte: u8) {
        self.output.borrow_mut().push(byte as char);
    }
}

/// 0-cost InputOutput implementor.
/// Intended for benchmarking.
/// `getch()` will panic, and `print()` will be ignored.
#[derive(Clone)]
pub struct NoIO {}
impl InputOutput for NoIO {
    #[inline]
    fn getch(&self) -> u8 {
        panic!("No Input supported!")
    }

    #[inline]
    fn print(&self, _: u8) {
        // we don't have to panic here, just do nothing
    }
}
