// rust_debug

#![allow(dead_code)]

use std::io::*;

pub fn flush_out() {
    let _ = std::io::stdout().flush();
}

pub fn break_here<F>(pred:bool, f:F) where F:Fn() {
    if pred {
        f();
        print!("\n  break: press return to continue");
        flush_out();
        let mut _buff = String::new();
        let _ = std::io::stdin().read_line(&mut _buff);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
