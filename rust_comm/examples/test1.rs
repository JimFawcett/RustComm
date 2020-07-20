/////////////////////////////////////////////////////////////
// rust_comm::test1.rs - Test Tcp Communation Library      //
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

fn main() {

    print!("\n  -- Demo rust_comm --\n");
    
    let addr = "127.0.0.1:8080";
    let mut lsnr = Listener::<P,Log>::new();
    let rslt = lsnr.start(addr);
    if rslt.is_err() {
        return;
    }
    let handle = rslt.unwrap();
    
    let rslt = Connector::<P,M,Log>::new(addr);
    if rslt.is_ok() {
        let conn = rslt.unwrap();

        let mut msg = Message::new();
        msg.set_type(MessageType::TEXT);
        msg.set_body_str("message #1");
        print!("\n  main posting msg: {:?}", msg.get_body_str());
        conn.post_message(msg);
        let msg = conn.get_message();
        print!("\n  main received msg: {:?}",msg.get_body_str());
    
        let mut msg = Message::new();
        msg.set_type(MessageType::TEXT);
        msg.set_body_str("message #2");
        print!("\n  main posting msg: {:?}", msg.get_body_str());
        conn.post_message(msg);
        let msg = conn.get_message();
        print!("\n  main received msg: {:?}",msg.get_body_str());

        /*-- shut down connertor --*/
        let mut msg = Message::new();
        msg.set_type(MessageType::END);
        print!("\n  main posting {:?} msg", "END");
        conn.post_message(msg);
    }

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