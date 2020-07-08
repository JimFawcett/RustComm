///////////////////////////////////////////////////////////////
// rust_comm::lib.rs - Prototype Tcp Communication System    //
//                                                           //
// Jim Fawett, https://JimFawcett.github.io, 08 July 2020    //
///////////////////////////////////////////////////////////////
/*
   rust_comm::lib.rs Facilities:
  ---------------------------------
   This file provides a library for a Prototype Tcp Communicator.

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
   - Support interhangeably Messages that use references to
     external facilities for defining message body contents
     rather than packing into message. 
*/
#![allow(dead_code)]

use std::fmt::*;
// use std::thread::*;

/*-------------------------------------------------------------
  Message Class
*/
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MessageType {
    msgtype: u8,
}
impl MessageType {
    pub const TEXT:u8 = 1;
    pub const BYTES:u8 = 2;
    pub const END:u8 = 4;

    pub fn get_type(&self) -> u8 {
        self.msgtype
    }
    pub fn set_type(&mut self, mt:u8) {
        if (mt == MessageType::TEXT) | 
           (mt == MessageType::BYTES) | 
           (mt == MessageType::END) {
            self.msgtype = mt;
        }
    }
}

#[derive(Debug, Clone)]
pub struct Message {
    mt: MessageType,
    body_buffer: Vec<u8>,
}
impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "{{\n    MessageType: {:?}\n    body size: {}\n  }}", 
            self.type_display(), self.body_buffer.len()
        )
    } 
}
impl Message {
    pub fn new() -> Message {
        Message {
            mt: MessageType { msgtype: 1, },
            body_buffer: Vec::<u8>::new(),
        }
    }
    pub fn set_type(&mut self, mt: u8) {
        self.mt.set_type(mt);
    }
    pub fn get_type(&self) -> MessageType {
        self.mt
    }
    pub fn set_body_bytes(&mut self, b:Vec<u8>) {
        self.body_buffer = b;
    }
    pub fn set_body_str(&mut self, s: &str) {
        let buff = s.as_bytes();
        for byte in buff {
            self.body_buffer.push(*byte);
        }
    }
    pub fn get_body_size(&self) -> usize {
        self.body_buffer.len()
    }
    pub fn get_body(&self) -> &Vec<u8> {
        &self.body_buffer
    }
    pub fn get_body_str(&self) -> String {
        let s = String::from_utf8_lossy(&self.body_buffer);
        s.to_string()
    }
    pub fn clear(&mut self) {
        self.body_buffer.clear();
    }
    fn type_display(&self) -> String {
        let typ:u8 = self.mt.get_type();
        let mut s = String::new();
        if typ == MessageType::TEXT {
            s.push_str("TEXT");
        }
        else if typ == MessageType::BYTES {
            s.push_str("BYTES");
        }
        else {
            s.push_str("END");
        }
        s
    }
}
/*---------------------------------------------------------
  External helper functions
*/
pub fn bytes_to_vec(ba: &[u8]) -> Vec<u8> {
    let v:Vec<u8> = ba.iter().cloned().collect();
    v
}

pub fn show_msg(msg: &Message) {
    print!("\n  {}", msg);
}

pub fn show_body(msg: &Message) {
    print!("\n  {:?}", msg.get_body());
}

pub fn show_body_str(msg: &Message) {
    let s = msg.get_body_str();
    print!("\n  {:?}", s);
}

/*-------------------------------------------------------------
  Sender - Message-passing Comm
*/
use std::io::{Read, Write, ErrorKind};

#[derive(Debug)]
pub struct Sender {
    //buf_stream: std::io::BufWriter<std::net::TcpStream>
    stream_opt: std::option::Option<std::net::TcpStream>,
}
impl Sender {
    pub fn new() -> Sender {
        Sender {
            stream_opt: None,
        }
    }
    pub fn connect(&mut self, addr: &str) -> std::io::Result<()> {
        self.stream_opt = Some(std::net::TcpStream::connect(addr)?);
        // print!("\n  -- leaving Sndr::connect --");
        Ok(())
    }
    pub fn send_message(&mut self, msg:Message) -> std::io::Result<()> {
        // print!("\n  -- entered send_message --");
        let rslt = &self.stream_opt;
        let mut stream : &std::net::TcpStream;
        match rslt {
            Some(strm) => stream = strm,
            None => return Err(std::io::Error::new(ErrorKind::Other,"")),
        }
        let typebyte = msg.get_type().get_type();
        let buf = [typebyte];
        stream.write(&buf)?;
        let bdysz = msg.get_body_size();
        /*-- to_ne_bytes() converts integral type to byte array --*/
        stream.write(&bdysz.to_ne_bytes())?;
        stream.write(&msg.get_body())?;
        print!("\n  -- send_message succeeded --");
        Ok(())
    }
}
/*---------------------------------------------------------
  Receiver - Message-passing Comm
*/
pub struct Receiver {
    tcpl: std::net::TcpListener,
}
impl Receiver {
    pub fn new(addr: &str) -> Receiver {
        Receiver {
            tcpl: std::net::TcpListener::bind(addr).unwrap(),
        }
    }
    pub fn start(&mut self) -> std::io::Result<()> {
        print!("\n  -- starting Receiver --");
        for stream in self.tcpl.incoming() {
            // print!("\n  -- entered incoming loop --");
            let rslt = handle_client(&mut stream?);
            match rslt {
                Ok(_) => {
                    print!("\n---- handle_client io exit --");
                },
                Err(_error) => { 
                    print!("\n  -- terminating: END message --");
                    break 
                },
            }
        }
        Ok(())
    }
}
// /*---------------------------------------------------------
//   External function that defines client processing 
//   for each Receiver connection.
// */
// #[allow(unreachable_code)]
// pub fn handle_client(stream: &std::net::TcpStream) -> std::io::Result<()> {
//     // print!("\n  -- entered handle_client --");
//     let mut reader = std::io::BufReader::new(stream.try_clone()?);
//     loop {
//         let mut msg = Message::new();
//         /*-- get MessageType --*/
//         let buf = &mut [0u8; 1];
//         reader.read_exact(buf)?;
//         let msgtype = buf[0];
//         msg.set_type(msgtype);
//         /*-- get body size --*/
//         let buf = &mut [0u8; 4];
//         reader.read_exact(buf)?;
//         let bdysz = usize::from_ne_bytes(*buf);
//         /*-- get body bytes --*/
//         let mut bdy = vec![0u8;bdysz];
//         reader.read_exact(&mut bdy)?;
//         msg.set_body_bytes(bdy);
//         print!("\n  -- received message --");
//         // show_msg(&msg);
//         if msg.get_body_size() > 0 {
//             show_body_str(&msg);
//             // show_body(&msg);
//         }
//         if msg.get_type().get_type() == MessageType::END {
//             // print!("\n  -- returning from handle client END message --");
//             let error = std::io::Error::new(ErrorKind::Other, "END");
//             return Err(error);
//         }
//     }
//     assert_eq!(1,2);  // should never get here
// }
impl Receiver {
    /*-- creates and starts Receiver listening on addr --*/
    fn do_rcv(addr: &str) {
        let mut rcvr = Receiver::new(addr);
        let rslt = rcvr.start();
        match rslt {
            Ok(_) => {}, //print!("\n---- normal return from rcvr start ----"),
            Err(_) => print!("\n----error return from rcvr start ----"),
        }
    }
    /*-- starts listener running on a dedicated thread --*/
    pub fn start_listener(addr: &'static str) -> std::thread::JoinHandle<()> {
        let handle = std::thread::spawn(move || {
            Receiver::do_rcv(addr);
        });
        handle
    }
    /*-- factors out result processing for connections --*/
    pub fn check_connection(rslt: &std::io::Result<()>) {
        match rslt {
            Ok(_) => print!("\n  -- connection successful --"),
            Err(_) => print!("\n---- connection failed ----"),
        }
    }
    /*-- factors out result processing for io operations --*/
    pub fn check_io(rslt: &std::io::Result<()>) {
        match rslt {
            Ok(_) => {},
            Err(_) => print!("\n---- io failed ----"),
        }
    }
}
/*---------------------------------------------------------
  External function that defines client processing 
  for each Receiver connection.
*/
#[allow(unreachable_code)]
pub fn handle_client(stream: &std::net::TcpStream) -> std::io::Result<()> {
    // print!("\n  -- entered handle_client --");
    let mut reader = std::io::BufReader::new(stream.try_clone()?);
    loop {
        let mut msg = Message::new();
        /*-- get MessageType --*/
        let buf = &mut [0u8; 1];
        reader.read_exact(buf)?;
        let msgtype = buf[0];
        msg.set_type(msgtype);
        /*-- get body size --*/
        let buf = &mut [0u8; 4];
        reader.read_exact(buf)?;
        let bdysz = usize::from_ne_bytes(*buf);
        /*-- get body bytes --*/
        let mut bdy = vec![0u8;bdysz];
        reader.read_exact(&mut bdy)?;
        msg.set_body_bytes(bdy);
        print!("\n  -- received message --");
        // show_msg(&msg);
        if msg.get_body_size() > 0 {
            show_body_str(&msg);
            // show_body(&msg);
        }
        if msg.get_type().get_type() == MessageType::END {
            // print!("\n  -- returning from handle client END message --");
            let error = std::io::Error::new(ErrorKind::Other, "END");
            return Err(error);
        }
    }
    assert_eq!(1,2);  // should never get here
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
