// rust_comm_processing::test1.rs

#![allow(unused_imports)]
use std::net::{TcpListener, TcpStream};

use rust_traits::*;
use rust_comm_processing::*;
use rust_message::*;
use rust_comm_logger::*;

fn construction() {
    let msg = Message::new();
    let _cp = CommProcessing::<VerboseLog>::new();
    let addr = "127.0.0.1:8080";
    let _lstnr = TcpListener::bind(addr);
    let mut stream = TcpStream::connect(addr).unwrap();
    let _ = CommProcessing::<VerboseLog>::send_message(msg, &mut stream);
}

fn main() {

    construction();

    print!("\n  That's all Folks!\n\n");
}