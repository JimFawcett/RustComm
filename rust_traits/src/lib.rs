/////////////////////////////////////////////////////////////
// comm_traits.rs - traits module                          //
//                                                         //
// Jim Fawcett, https://JimFawcett.github.com, 20 Jul 2020 //
/////////////////////////////////////////////////////////////
/*
   Defines traits used for rust_comm:
   - Logger
   - MsgType
   - Msg
   - Sndr<M>
   - Rcvr<M>
   - Process<M>
*/

use std::net::{TcpStream};
use std::io::{BufReader, BufWriter, Result};
use std::str::Utf8Error;
// use rust_blocking_queue::*;

// pub const MSG_SIZE:usize = 4096;

pub trait Logger : Send {
    fn write(msg: &str);
}

pub trait MsgType : Send + std::fmt::Debug {
    fn get_type(&self) -> u8;
    fn set_type(&mut self, mt:u8);
}

pub trait Msg : Send + std::fmt::Debug {
    fn new(sz:usize) -> Self;
    fn init(&mut self);
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn set_type(&mut self, mt:u8);
    fn get_type(&self) -> u8;
    fn set_content_bytes(&mut self, buff: &[u8]);
    fn get_content_bytes(&self) -> &[u8];
    fn set_content_str(&mut self, s: &str);
    fn get_content_str(&self) -> std::result::Result<&str, Utf8Error>;
    fn show_message(&self, fold:usize);
    fn set_content_size(&mut self, sz:usize);
    fn get_content_size(&self) -> usize;
    fn set_bytes(&mut self, buff:&[u8]);
    fn get_bytes(&self) -> &[u8];
    fn get_mut_bytes(&mut self) -> &mut [u8];
    fn get_ref(&self) -> &Vec<u8>;
    fn get_mut_ref(&mut self) -> &mut Vec<u8>;
    fn type_display(&self) -> String;
}
pub trait Sndr<M> : Send 
where M: Msg + Clone + Send + Default,
{
    fn send_message(msg: &M, stream: &mut TcpStream) -> Result<()>;
    fn buf_send_message(msg: &M, stream: &mut BufWriter<TcpStream>) -> Result<()>;
}
pub trait Rcvr<M>: Send 
where M: Msg + Clone + Send + Default,
{
    fn recv_message(stream: &mut TcpStream) -> Result<M>;
    fn buf_recv_message(stream: &mut BufReader<TcpStream>) -> Result<M>;
}
pub trait Process<M> : Send 
where M: Msg + Clone + Send + Default,
{
    fn process_message(m: &mut M);
}
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
