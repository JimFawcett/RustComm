/////////////////////////////////////////////////////////////
// rust_comm_processing::lib.rs - Application Specific     //
//                                                         //
// Jim Fawcett, https://JimFawcett.github.io, 19 Jul 2020  //
/////////////////////////////////////////////////////////////
/*
   CommProcessing<L>:
   - defines send_message, recv_message, and process_message
   - each of these needs to be tailored to the specifics of
     the Message class
*/

#![allow(unused_imports)]
#![allow(dead_code)]

/*-- RustComm facilities --*/
use rust_traits::*;
use rust_message::*;
//use rust_blocking_queue::*;
use rust_comm_logger::*;

/*-- std library facilities --*/
use std::fmt::*;
use std::net::{TcpStream};
use std::io::prelude::*;
use std::io::{BufReader, BufWriter, Write};
use std::convert::{TryInto};

type M = Message;

/*---------------------------------------------------------
  CommProcessing<L> 
  - defines application specific processing for the
    appliczation's message type
  - L is a logger type the must implement the Logger trait
*/
#[derive(Debug, Copy, Clone, Default)]
pub struct CommProcessing<L>
where L: Logger + Debug + Copy + Clone + Default {
    log: L,
}
impl<L> CommProcessing<L>
where L: Logger + Debug + Copy + Clone + Default
{
    pub fn new() -> CommProcessing<L> {
        CommProcessing {
            log: L::default(),
        }
    }
}
impl<M,L> Sndr<M> for CommProcessing<L>
where 
    M: Msg + Clone + Send + Default,
    L: Logger + Debug + Copy + Clone + Default
{
    fn send_message(msg: &M, stream: &mut TcpStream) -> std::io::Result<()>
    {
        L::write(&format!("\n  msg.len(): {}", msg.len()));
        stream.write(&msg.get_ref())?;
        Ok(())
    }
    fn buf_send_message(msg: &M, stream: &mut BufWriter<TcpStream>) -> std::io::Result<()>
    {
        L::write(&format!("\n  msg.len(): {}", msg.len()));
        stream.write(&msg.get_ref())?;
        let msg_type = msg.get_type(); 
        if msg_type == MessageType::FLUSH as u8 
            || msg_type == MessageType::END as u8 
            || msg_type == MessageType::QUIT as u8 
        {
            L::write("\n  flushing stream");
            let _ = stream.flush();
        }
        Ok(())
    }
}
impl<M,L> Rcvr<M> for CommProcessing<L>
where 
    M: Msg + Clone + Send + Default,
    L: Logger + Debug + Copy + Clone + Default
{
    /*-- reads message and enques in supplied BlockingQueue<M> --*/
    fn recv_message(stream: &mut TcpStream) -> std::io::Result<M> 
    {
        L::write("\n  attempting to receive msg in commProc");
        let buf = &mut [0u8; HEADER_SIZE];
        stream.read_exact(buf)?;
        let msgtype = buf[0];
        let sz_slice = &buf[1..HEADER_SIZE];
        let mut dst = [0u8;8];
        dst.clone_from_slice(sz_slice); // array from byte slice
        let bdysz = usize::from_be_bytes(dst);   // usize from byte array

        let mut bdy = vec![0u8;bdysz];
        stream.read_exact(&mut bdy)?;        
        let msg_size = TYPE_SIZE + CONTENT_SIZE + bdysz;
        let mut msg = M::new(msg_size);
        msg.set_type(msgtype);
        msg.set_content_bytes(&bdy);
        Ok(msg)
    }
    /*-- same as above but uses buffered reader --*/
    fn buf_recv_message(stream: &mut BufReader<TcpStream>) -> std::io::Result<M> 
    {
        L::write("\n  attempting to receive msg in commProc");
        let buf = &mut [0u8; HEADER_SIZE];
        stream.read_exact(buf)?;
        let msgtype = buf[0];
        let sz_slice = &buf[1..HEADER_SIZE];
        let mut dst = [0u8;8];
        dst.clone_from_slice(sz_slice); // array from byte slice
        let bdysz = usize::from_be_bytes(dst);   // usize from byte array

        let mut bdy = vec![0u8;bdysz];
        stream.read_exact(&mut bdy)?;        
        let msg_size = TYPE_SIZE + CONTENT_SIZE + bdysz;
        let mut msg = M::new(msg_size);
        msg.set_type(msgtype);
        msg.set_content_bytes(&bdy);
        Ok(msg)
    }
}
/*---------------------------------------------------------
  Process<M> handles processing of each message on 
  Listener<P,L>
*/
impl<M,L> Process<M> for CommProcessing<L>
where 
    M: Msg + Clone + Send + Default,
    L: Logger + Debug + Copy + Clone + Default
{
    fn process_message(msg: &mut M) 
    {
        L::write("\n--entered process_message--");
        let msg_type = msg.get_type();
        if msg_type != MessageType::FLUSH as u8 
            && msg_type != MessageType::END as u8 
            && msg_type != MessageType::QUIT as u8 
        {
            msg.set_type(MessageType::REPLY as u8);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn construction() {
        let msg = Message::new(64);
        let _cp = CommProcessing::<MuteLog>::default();
        let addr = "127.0.0.1:8080";
        let _lstnr = std::net::TcpListener::bind(addr);
        let mut stream = std::net::TcpStream::connect(addr).unwrap();
        let _ = CommProcessing::<MuteLog>::send_message(&msg, &mut stream);
        assert_eq!(2 + 2, 4);
    }
}
