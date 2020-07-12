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
   - Uses unqueued full-duplex message sending and receiving
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
   - Provide mechanism to shut down Receiver gracefully
   - Support interhangeably Messages that use references to
     external facilities for defining message body contents
     rather than packing into message. 
*/
#![allow(dead_code)]

use std::fmt::*;
// use std::time::{Duration};

pub trait Log {
    fn write(msg: &str);
    fn default() -> Self;
}
pub struct MuteLog;
impl Log for MuteLog {
    fn write(_msg: &str) {}
    fn default() -> MuteLog {
        MuteLog {}
    }
}
impl MuteLog {
    pub fn new() -> MuteLog {
        MuteLog {}
    }
}
pub struct VerboseLog;
impl Log for VerboseLog {
    fn write(msg: &str) {
        print!("{}", msg);
    }
    fn default() -> VerboseLog {
        VerboseLog {}
    }
}
impl VerboseLog {
    pub fn new() -> VerboseLog {
        VerboseLog {}
    }
}
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
    pub const QUIT:u8 = 8;

    pub fn get_type(&self) -> u8 {
        self.msgtype
    }
    pub fn set_type(&mut self, mt:u8) {
        if (mt == MessageType::TEXT) | 
           (mt == MessageType::BYTES) | 
           (mt == MessageType::END) |
           (mt == MessageType::QUIT) {
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
use std::io::{Read, Write, Error, ErrorKind, BufReader, BufWriter};
use std::net::{TcpStream};

#[derive(Debug)]
pub struct Sender<L> where L: Log {
    //buf_stream: std::io::BufWriter<std::net::TcpStream>
    stream_opt: std::option::Option<TcpStream>,
    // stream_opt: std::option::Option<std::net::TcpStream>,
    log: L,
}
impl<L> Sender<L> where L:Log {
    pub fn new() -> Sender<L> {
        Sender::<L> {
            stream_opt: None,
            log: L::default(), 
        }
    }
    pub fn connect(&mut self, addr: &str) -> std::io::Result<()> {
        self.stream_opt = Some(TcpStream::connect(addr)?);
        // print!("\n  -- leaving Sndr::connect --");
        Ok(())
    }
    pub fn send_message(&mut self, msg:Message) -> std::io::Result<()> {
        let rslt = &self.stream_opt;
        let stream: &TcpStream;
        match rslt {
            Some(strm) => stream = strm,
            None => return Err(std::io::Error::new(ErrorKind::Other,"")),
        }
        let mut buf_stream = BufWriter::new(stream);
        let typebyte = msg.get_type().get_type();
        let buf = [typebyte];
        buf_stream.write(&buf)?;
        let bdysz = msg.get_body_size();
        /*-- to_be_bytes() converts integral type to big-endian byte array --*/
        buf_stream.write(&bdysz.to_be_bytes())?;
        buf_stream.write(&msg.get_body())?;
        let _ = buf_stream.flush();
        Ok(())
    }
    pub fn recv_message(&mut self) -> std::io::Result<()> {
        let rslt = &self.stream_opt;
        let stream: &TcpStream;
        match rslt {
            Some(strm) => stream = strm,
            None => return Err(std::io::Error::new(ErrorKind::Other,"")),
        }
        let mut buf_stream = BufReader::new(stream);
        
        let mut msg = Message::new();
        /*-- get MessageType --*/
        let buf = &mut [0u8; 1];
        buf_stream.read_exact(buf)?;
        let msgtype = buf[0];
        msg.set_type(msgtype);
        /*-- get body size --*/
        let buf = &mut [0u8; 4];
        buf_stream.read_exact(buf)?;
        let bdysz = usize::from_be_bytes(*buf);
        /*-- get body bytes --*/
        let mut bdy = vec![0u8;bdysz];
        buf_stream.read_exact(&mut bdy)?;
        msg.set_body_bytes(bdy);
        let mut mod_body = msg.get_body_str();
        mod_body.push_str(" reply");
        msg.clear();
        msg.set_body_str(&mod_body);
        L::write("\n  -- Sender received reply message --");
        // show_msg(&msg);
        if msg.get_body_size() > 0 {
            show_body_str(&msg);
            // show_body(&msg);
        }
        Ok(())
    }
    /*-- factors out result processing for connections --*/
    pub fn check_connection(rslt: &std::io::Result<()>) {
        match rslt {
            Ok(_) => L::write("\n  -- connection successful --"),
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
  Receiver - Message-passing Comm
*/
use std::sync::{Arc, atomic::AtomicBool, atomic::Ordering};

#[derive(Debug)]
pub struct Receiver<L> where L:Log + Send + 'static {
    run: Arc<AtomicBool>,
    listen_addr: String,
    log: L,
}
impl<L> Clone for Receiver<L> where L:Log + Send + 'static {
    fn clone(&self) -> Receiver<L> {
        Receiver {
            run : Arc::clone(&self.run),
            listen_addr: self.listen_addr.clone(),
            log: L::default(),
        }
    }
}
impl<L> Receiver<L> where L:Log + Send + 'static {
    pub fn new(addr: &str) -> Receiver<L> {
        Receiver {
            run: Arc::new(AtomicBool::new(true)),
            listen_addr: addr.to_string(),
            log: L::default(),
        }
    }
    pub fn stop(&mut self) {
        self.run.store(false, Ordering::Relaxed);
        L::write("\n  -- stopping Receiver --");
    }
    pub fn start(&mut self) -> std::io::Result<()> {
        L::write("\n  -- starting Receiver --");
        let tcpl = std::net::TcpListener::bind(&self.listen_addr)?;
        for stream in tcpl.incoming() {
            let run = self.run.load(Ordering::Relaxed);
            if !run {
                L::write("\n  -- breaking out of incoming loop --");
                break;
            }
            let _handle = std::thread::spawn(move || {
                let rslt = handle_client::<L>(&mut stream.unwrap());
                match rslt {
                    Ok(_) => {
                        let _ = std::io::stdout().flush();
                        return;
                    },
                    Err(_error) => { 
                        if _error.to_string() == "END" {
                            let _ = std::io::stdout().flush();
                            return; 
                        }
                        else if _error.to_string() == "QUIT" {
                            L::write("\n  -- terminating: QUIT message --");
                            let _ = std::io::stdout().flush();
                            return; 
                        }
                    },
                }    
            });
            std::thread::yield_now();
        }
        let error = Error::new(ErrorKind::Other, "QUIT");
        Err(error)
    }
    /*-- creates and starts Receiver listening on addr --*/
    fn do_rcv(&mut self) {
        let rslt = self.start();
        match rslt {
            Ok(_) => { 
                /*print!("\n---- normal return from rcvr start ----"); */
            },
            Err(_) => { 
                /*print!("\n----error return from rcvr start ----");*/ 
            },
        }
    }
    /*-- starts listener running on a dedicated thread --*/
    pub fn start_listener(&mut self) -> std::thread::JoinHandle<()> {
        let mut cln = self.clone();
        let handle = std::thread::spawn(move || {
            cln.do_rcv();
        });
        handle
    }
}
/*-- implement Receiver associated functions --*/
impl<L> Receiver<L> where L:Log + Send {

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
pub fn handle_client<L:Log> (stream: &std::net::TcpStream) -> std::io::Result<()> {

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
        let bdysz = usize::from_be_bytes(*buf);
        /*-- get body bytes --*/
        let mut bdy = vec![0u8;bdysz];
        reader.read_exact(&mut bdy)?;
        msg.set_body_bytes(bdy);
        L::write("\n  -- Listener received message --");
        // show_msg(&msg);
        if msg.get_body_size() > 0 {
            show_body_str(&msg);
            // show_body(&msg);
        }
        /*-- send echo reply --*/
        send_message(stream, msg.clone())?;

        if msg.get_type().get_type() == MessageType::END {
            let error = std::io::Error::new(ErrorKind::Other, "END");
            return Err(error);
        }
        if msg.get_type().get_type() == MessageType::QUIT {
            let error = std::io::Error::new(ErrorKind::Other, "QUIT");
            return Err(error);
        }
    }
    assert_eq!(1,2);  // should never get here
}
pub fn send_message(stream: &std::net::TcpStream, msg:Message) 
    -> std::io::Result<()> {
    let mut buf_stream = BufWriter::new(stream);
    let typebyte = msg.get_type().get_type();
    let buf = [typebyte];
    buf_stream.write(&buf)?;
    // let mut body = msg.get_body_str();
    // body.push_str(" reply");
    // new_msg.clear();
    // new_msg.set_body_str(&body);
    let bdysz = msg.get_body_size();
    /*-- to_be_bytes() converts integral type to big-endian byte array --*/
    buf_stream.write(&bdysz.to_be_bytes())?;
    buf_stream.write(&msg.get_body())?;
    let _ = buf_stream.flush();
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
