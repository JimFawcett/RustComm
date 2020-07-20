// rust_comm_logger::lib.rs

use rust_traits::*;
use std::fmt::*;

#[derive(Debug, Copy, Clone, Default)]
pub struct Logr<L: Logger> {
    _log: L,
}
impl<L> Logger for Logr<L> where L: Logger + Debug + Copy + Clone + Default {
   fn write(msg: &str) {
        L::write(msg);
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct MuteLog {}
impl Logger for MuteLog {
    fn write(_msg: &str) {
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct VerboseLog {}
impl Logger for  VerboseLog {
    fn write(msg: &str) {
        print!("{}", msg);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
