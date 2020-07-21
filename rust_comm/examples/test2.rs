/////////////////////////////////////////////////////////////
// rust_comm::test2.rs - Test Tcp Communation Library      //
//                                                         //
// Jim Fawcett, https://JimFawcett.github.io, 19 Jul 2020  //
/////////////////////////////////////////////////////////////
/*
   Demo:
   - start Listener component
   - start Connector component
   - send a few messages, ,observe replies
   - send END message to exit client handler
   - send QUIT message to shut down Listener
*/
#![allow(unused_imports)]
#![allow(dead_code)]

use std::io::prelude::*;

/*-- component library rust_blocking_queue --*/
use rust_message::*;
use rust_traits::*;
use rust_comm_processing::*;
use rust_comm_logger::*;
use rust_comm::*;

type Log = MuteLog;
type M = Message;
type P = CommProcessing<Log>;

fn start_client(
       addr: &'static str, name: &'static str, n:usize, sd:bool
   ) -> std::thread::JoinHandle<()> {
    
    let handle = std::thread::spawn(move || {
        let conn = Connector::<P,M,Log>::new(addr).unwrap();
        for i in 0..n {
            /*-- used to test error handling --*/
            if sd && i == n-1 {
                let mut msg = Message::new();
                msg.set_type(MessageType::SHUTDOWN);
                conn.post_message(msg);
                return;
            }
            /*---------------------------------*/
            let mut msg = Message::new();
            let s = format!("msg #{} from {}", i, name);
            msg.set_body_str(&s);
            print!("\n  posting msg:  {:?}", s);
            conn.post_message(msg);
            let msg = conn.get_message();
            print!("\n  received msg: {:?}", msg.get_body_str());
        }
        let mut msg = Message::new();
        msg.set_type(MessageType::END);
        conn.post_message(msg);
    });
    handle
}

fn main() {

    print!("\n  -- test2: rust_comm --\n");
    
    let addr = "127.0.0.1:8080";
    let mut lsnr = Listener::<P,Log>::new();
    let rslt = lsnr.start(addr);
    if rslt.is_err() {
        return;
    }
    let handle = rslt.unwrap();

    let h1 = start_client(addr, "bugs ", 5, false);
    let h2 = start_client(addr, "elmer", 5, false);
    let h3 = start_client(addr, "daffy", 5, false);
    
    let _ = h1.join();
    let _ = h2.join();
    let _ = h3.join();

    /*-- shut down listener --*/
    let conn = Connector::<P,M,Log>::new(addr).unwrap();
    let mut msg = Message::new();
    msg.set_type(MessageType::QUIT);
    print!("\n  main posting {:?} msg", "QUIT");
    conn.post_message(msg);
    let _ = std::io::stdout().flush();

    let _ = handle.join();
    println!();
}