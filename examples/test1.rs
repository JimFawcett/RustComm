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
   Provides two user defined types: Sender and Receiver. 
   - Uses unbuffered, unqueued full-duplex message sending and 
     receiving
   - Each message has a fixed size header and Vec<u8> body.
   - For each Sender connection, Receiver processes messages
     until receiving a message with MessageType::END.
   - Receiver spawns a thread for each client connection and
     processes messages in an external handle_client function.
   - In this version, handle_client only displays message body
     to console.  It does not send back a replay message.

   Expected Changes and Additions:
  ---------------------------------
   - Add reply messages to this demo.
   - Add Sender queue and a threadpool in Receiver
   - Convert to buffered reads and writes
   - Add user-defined Comm type that composes a Sender and a
     Receiver.   
*/
#![allow(dead_code)]

use rust_comm::*;

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
fn start_client(addr:&'static str, name:&'static str, n:u32) {
  let _handle = std::thread::spawn(move || {
    let mut sndr = Sender::new();
    Receiver::check_connection(&sndr.connect(addr));
  
    for i in 0..n {
        let mut msg = Message::new();
        // msg.set_name("bugs");
        let bstr = format!("msg #{} from client {}", i, name);
        msg.set_body_str(&bstr);
        Receiver::check_io(&sndr.send_message(msg));
    }
    /*-- send END message --*/
    let mut msg = Message::new();
    msg.set_type(MessageType::END);
    Receiver::check_io(&sndr.send_message(msg));
    });
}

fn test_comm() {

    print!("\n  -- test comm --");
    println!();

    let addr = "127.0.0.1:8081";
    let handle = Receiver::start_listener(addr);

    start_client(addr, "bugs", 5);
    start_client(addr, "elmer", 5);
    start_client(addr, "daffy", 5);

    let _ = handle.join();
}
fn main() {

    // test_message();
    // println!();
    test_comm();

    print!("\n\n  That's all Folks!\n\n");
}