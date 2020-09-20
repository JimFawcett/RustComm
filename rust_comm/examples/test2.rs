/////////////////////////////////////////////////////////////
// rust_comm::test2.rs - Test Tcp Communication Library    //
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

fn start_client(
       addr: &'static str, name: &'static str, n:usize, sd:bool
   ) -> std::thread::JoinHandle<()> {
    
    let handle = std::thread::spawn(move || {
        let conn = Connector::<P,M,Log>::new(addr).unwrap();
        print!("\n  connection succeeded");
        for i in 0..n {
            /*-- used to test error handling --*/
            if sd && i == n-1 {
                let mut msg = Message::new(TYPE_SIZE + CONTENT_SIZE);
                msg.set_type(MessageType::QUIT as u8);
                conn.post_message(msg);
                return;
            }
            /*---------------------------------*/
            let s = format!("msg #{} from {}", i, name);
            let mut msg = Message::create_msg_str_fit(&s);
            msg.set_type(MessageType::FLUSH as u8);
            print!("\n  posting msg:  {:?}", s);
            Log::write(&format!("\n  message size: {:?}", msg.len()));
            conn.post_message(msg);
            let msg = conn.get_message();
            print!("\n  received msg: {:?}", msg.get_content_str().unwrap());
        }
        let mut msg = Message::new(TYPE_SIZE + CONTENT_SIZE);
        msg.set_type(MessageType::END as u8);
        print!("\n  posting END message");
        conn.post_message(msg);
    });
    handle
}

fn main() {

    print!("\n  -- test2: rust_comm\n  -- variable size msgs, buffered\n");
    
    let addr = "127.0.0.1:8080";
    let mut lsnr = Listener::<P,Log>::new(8);
    let rslt = lsnr.start(addr);
    if rslt.is_err() {
        print!("\n  can't start listener on {:?}", addr);
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
    lsnr.stop();
    let _ = handle.join();
    println!();
}