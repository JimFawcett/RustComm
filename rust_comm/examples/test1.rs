/////////////////////////////////////////////////////////////
// rust_comm::test1.rs - Test Tcp Communication Library    //
//   - RustComm_VariableSizeMsg                            //
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

// const MESS_SIZE:usize = 64;

fn main() {

    print!("\n  -- test1: rust_comm\n  -- variable size msgs, buffered\n");
    
    let addr = "127.0.0.1:8080";
    let mut lsnr = Listener::<P,Log>::new(8);
    let rslt = lsnr.start(addr);
    if rslt.is_err() {
        return;
    }
    let handle = rslt.unwrap();
    
    let rslt = Connector::<P,M,Log>::new(addr);
    if rslt.is_ok() {
        print!("\n  connected to: {:?}",addr);
        let _ = std::io::stdout().flush();
        let conn = rslt.unwrap();
        let mut msg = Message::create_msg_str_fit("message #1");
        msg.set_type(MessageType::FLUSH as u8);
        print!("\n  main posting msg: {:?}", msg.get_content_str().unwrap());
        // let _ = std::io::stdout().flush();
        msg.show_message(8);
        let _ = std::io::stdout().flush();
        conn.post_message(msg);
        let msg = conn.get_message();
        print!("\n\n  main received msg: {:?}",msg.get_content_str().unwrap());
        let _ = std::io::stdout().flush();
    
        let mut msg = Message::create_msg_str_fit("message #2");
        msg.set_type(MessageType::FLUSH as u8);
        print!("\n  main posting msg: {:?}", msg.get_content_str().unwrap());
        conn.post_message(msg);
        let msg = conn.get_message();
        print!("\n  main received msg: {:?}",msg.get_content_str().unwrap());

        /*-- shut down connector --*/
        let mut msg = Message::new(TYPE_SIZE + CONTENT_SIZE);
        msg.set_type(MessageType::END as u8);
        print!("\n  main posting {:?} msg", "END");
        conn.post_message(msg);
    }
    else {
        print!("\n  connection to {:?} failed", addr);
    }


    /*-- shut down listener --*/
    lsnr.stop();
    let _ = handle.join();
    println!();
}