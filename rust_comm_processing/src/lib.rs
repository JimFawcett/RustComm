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

#![allow(dead_code)]

/*-- RustComm facilities --*/
use rust_traits::*;
use rust_message::*;
use rust_blocking_queue::*;

/*-- std library facilities --*/
use std::fmt::*;
use std::net::{TcpStream};
use std::io::prelude::*;
use std::io::{BufReader, BufWriter, Write};

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
    M: Msg + std::fmt::Debug + Clone + Send + Default,
    L: Logger + Debug + Copy + Clone + Default
{
    fn send_message(msg: M, stream: &mut TcpStream) -> std::io::Result<()>
    {
        let typebyte = msg.get_type();
        let buf = [typebyte];
        stream.write(&buf)?;
        let bdysz = msg.get_body_size();
        /*-- to_be_bytes() converts integral type to big-endian byte array --*/
        stream.write(&bdysz.to_be_bytes())?;
        stream.write(&msg.get_body_bytes())?;
        let _ = stream.flush();
        Ok(())
    }
    fn buf_send_message(msg: M, stream: &mut BufWriter<TcpStream>) -> std::io::Result<()>
    {
        let typebyte = msg.get_type();
        let buf = [typebyte];
        stream.write(&buf)?;
        let bdysz = msg.get_body_size();
        /*-- to_be_bytes() converts integral type to big-endian byte array --*/
        stream.write(&bdysz.to_be_bytes())?;
        stream.write(&msg.get_body_bytes())?;
        let _ = stream.flush();
        Ok(())
    }
}
impl<M,L> Rcvr<M> for CommProcessing<L>
where 
    M: Msg + std::fmt::Debug + Clone + Send + Default,
    L: Logger + Debug + Copy + Clone + Default
{
    /*-- reads message and enques in supplied BlockingQueue<M> --*/
    fn recv_message(
        stream: &mut TcpStream, q:&BlockingQueue<M>
    ) -> std::io::Result<()> 
    {
        let mut msg = M::default();
        /*-- get MessageType --*/
        let buf = &mut [0u8; 1];
        stream.read_exact(buf)?;
        let msgtype = buf[0];
        msg.set_type(msgtype);
        /*-- get body size --*/
        let mut buf = [0u8; 8];
        stream.read_exact(&mut buf)?;
        let bdysz = usize::from_be_bytes(buf);
        /*-- get body bytes --*/
        let mut bdy = vec![0u8;bdysz];
        stream.read_exact(&mut bdy)?;
        msg.set_body_bytes(bdy);
        q.en_q(msg);
        Ok(())
    }
    /*-- same as above but uses buffered reader --*/
    fn buf_recv_message(
        stream: &mut BufReader<TcpStream>, 
        q: &BlockingQueue<M>
    ) -> std::io::Result<()> 
    {
        let mut msg = M::default();
        /*-- get MessageType --*/
        let buf = &mut [0u8; 1];
        stream.read_exact(buf)?;
        let msgtype = buf[0];
        msg.set_type(msgtype);
        /*-- get body size --*/
        let mut buf = [0u8; 8];
        stream.read_exact(&mut buf)?;
        let bdysz = usize::from_be_bytes(buf);
        /*-- get body bytes --*/
        let mut bdy = vec![0u8;bdysz];
        stream.read_exact(&mut bdy)?;
        msg.set_body_bytes(bdy);
        q.en_q(msg);
        Ok(())
    }
}
/*---------------------------------------------------------
  Process<M> handles processing of each message on 
  Listener<P,L>
*/
impl<M,L> Process<M> for CommProcessing<L>
where 
    M: Msg + std::fmt::Debug + Clone + Send + Default,
    L: Logger + Debug + Copy + Clone + Default
{
    fn process_message(m: M) -> M 
    {
        L::write("\n--entered process_message--");
        let mut msg = M::default();
        msg.set_type(MessageType::REPLY);
        let mut s = m.get_body_str();
        s.push_str(" reply");
        msg.set_body_str(&s);
        msg
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn construction() {
        let msg = Message::new();
        let _cp = CommProcessing::<Message>::default();
        let addr = "127.0.0.1:8080";
        let _lstnr = std::net::TcpListener::bind(addr);
        let mut stream = std::net::TcpStream::connect(addr).unwrap();
        let _ = CommProcessing::send_message(msg, &mut stream);
        assert_eq!(2 + 2, 4);
    }
}
