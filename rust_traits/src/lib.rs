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
use rust_blocking_queue::*;

pub trait Logger : Send {
    fn write(msg: &str);
}
pub trait MsgType : Send {
    fn get_type(&self) -> u8;
    fn set_type(&mut self, mt:u8);
}
pub trait Msg : Send {
    // fn new() -> &'static dyn Msg where Self: Sized;
    fn get_type(&self) -> u8;
    fn set_type(&mut self, mt:u8);
    fn set_body_bytes(&mut self, b:Vec<u8>);
    fn set_body_str(&mut self, s: &str);
    fn get_body_size(&self) -> usize;
    fn get_body_bytes(&self) -> &Vec<u8>;
    fn get_body_str(&self) -> String;
    fn clear(&mut self);
    fn type_display(&self) -> String;
    fn show_msg(&self);
}
pub trait Sndr<M> : Send 
where M: Msg + std::fmt::Debug + Clone + Send + Default,
{
    fn send_message(msg: M, stream: &mut TcpStream) -> Result<()>;
    fn buf_send_message(msg: M, stream: &mut BufWriter<TcpStream>) -> Result<()>;
}
pub trait Rcvr<M>: Send 
where M: Msg + std::fmt::Debug + Clone + Send + Default,
{
    fn recv_message(stream: &mut TcpStream, q:&BlockingQueue<M>) -> Result<()>;
    fn buf_recv_message(stream: &mut BufReader<TcpStream>, q:&BlockingQueue<M>) -> Result<()>;
}
pub trait Process<M> : Send 
where M: Msg + std::fmt::Debug + Clone + Send + Default,
{
    fn process_message(m: M) -> M;
}
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
