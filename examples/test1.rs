///////////////////////////////////////////////////////////////
// rust_comm::test1.rs - Prototype Tcp Communication System  //
//                                                           //
// Jim Fawett, https://JimFawcett.github.io, 08 July 2020    //
///////////////////////////////////////////////////////////////
/*
   rust_comm::test1.rs Facilities:
  ---------------------------------
   This file provides demonstration tests for Messages and for
   rust_comm::Sender and rust_comm::Receiver.

   rust_comm::lib.rs Facilities:
  -------------------------------
   Provides three user defined types: Message, Sender & Receiver. 
   - Uses unqueued full-duplex message sending and receiving
   - Each message has a fixed size header and Vec<u8> body.
   - For each Sender connection, Receiver processes messages
     until receiving a message with MessageType::END.
   - Receiver spawns a thread for each client connection and
     processes messages in an external handle_client function.
   - In this version, handle_client only displays message body
     to console.  It does not send back a reply message.

   Expected Changes and Additions:
  ---------------------------------
   - Add reply messages to this demo.
   - Add Sender queue and a threadpool in Receiver
   - Add user-defined Comm type that composes a Sender and a
     Receiver.   
*/
#![allow(dead_code)]

use rust_comm::*;
use std::thread::{JoinHandle};
use std::time::*;

// type Log = MuteLog;  // shows only messages
type Log = VerboseLog;  // shows messages and events

fn test_message() {

    print!("\n  -- test message type --");

    let mut msg = Message::new();
    print!("\n  msg: {}", msg);
    println!();

    msg.set_body_str("A B C");
    print!("\n  body: {}", msg);
    let s = msg.get_body_str();
    print!("\n  body: {:?}", s);
    println!();

    msg.clear();
    show_msg(&msg);
    println!();

    msg.set_body_bytes(bytes_to_vec(&[1,2,3]));
    msg.set_type(MessageType::BYTES);
    show_msg(&msg);
    show_body(&msg);
}
/*-- start client sending messages on dedicated thread --*/
fn start_client(addr:&'static str, name:&'static str, n:u32) 
   -> JoinHandle<()> {
  
  let _handle = std::thread::spawn(move || {
    let mut sndr = Sender::<Log>::new();
    Sender::<Log>::check_connection(&sndr.connect(addr));
  
    for i in 0..n {
        let mut msg = Message::new();
        let bstr = format!("msg #{} from client {}", i, name);
        msg.set_body_str(&bstr);
        Sender::<Log>::check_io(&sndr.send_message(msg));
        let _ = sndr.recv_message();
    }
    /*-- send END message --*/
    let mut msg = Message::new();
    msg.set_type(MessageType::END);
    Receiver::<Log>::check_io(&sndr.send_message(msg));
    });
    _handle
}

fn test_comm() {

    print!("\n  -- test comm --");
    println!();

    let addr = "127.0.0.1:8081";
    let mut rcvr = Receiver::<Log>::new(addr);
    let handle = rcvr.start_listener();

    let handle1 = start_client(addr, "bugs", 5);
    let handle2 = start_client(addr, "elmer", 5);
    let handle3 = start_client(addr, "daffy", 5);

    let _ = handle1.join();
    let _ = handle2.join();
    let _ = handle3.join();

    let millisecs = Duration::from_millis(200);
    std::thread::sleep(millisecs);
    
    rcvr.stop();
    // print!("\n  called stop()");

    let mut sndr = Sender::<Log>::new();
    Sender::<Log>::check_connection(&sndr.connect(addr));
    let mut msg = Message::new();
    msg.set_type(MessageType::QUIT);
    msg.set_body_str("Quit message");
    Sender::<Log>::check_io(&sndr.send_message(msg.clone()));
    
    let _ = handle.join();
}
fn main() {

    // test_message();
    // println!();
    test_comm();

    print!("\n\n  That's all Folks!\n\n");
}