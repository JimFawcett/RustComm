/*-----------------------------------------------
  Demonstrations of StopWatch, Timer, ...
*/
use rust_timer::{stop_watch, timer, date_time_stamp, date_stamp, time_stamp};

fn main() {
    print!("\n  === demo date_time_timer ===");
    println!();
    
    print!("\n  -- demo StopWatch --");
    stop_watch(25);
    println!();
    
    print!("\n  -- demo Timer --");
    print!("\n  starting timer(200)");
    let handle = timer(200);
    print!("\n  do some work while waiting for timer");
    let _ = handle.join(); 
    println!();
    
    print!("\n  -- demo DateTimeStamp --");
    print!("\n  now is:  {:?}", date_time_stamp());
    print!("\n  date is: {:?}", date_stamp());
    print!("\n  time is: {:?}", time_stamp());
    println!("\n\n  That's all Folks!\n\n");
}
