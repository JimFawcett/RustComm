// rust_debug::test1.rs

use rust_debug::*;

fn f() {
    print!("  -- here's a breakpoint");
}

fn main() {
    print!("\n  starting\n\n");
    for i in 1..5 {
        break_here(i == 3, f);
        let mut msg = String::from("step #");
        msg.push_str(&i.to_string());
        print!("    {:?}\n",msg);
    }
    print!("\n  finishing\n\n");
}