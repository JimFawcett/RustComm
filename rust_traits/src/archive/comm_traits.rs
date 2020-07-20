// comm_traits.rs - traits module

use std::net::{TcpStream};
use std::io::{BufReader, BufWriter, Result};

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
}
// pub trait Que<Msg> : Send {
//     fn en_q(&self, m: Msg);
//     fn de_q(&self) -> Msg;
//     fn len(&self) -> usize;
// }
pub trait Sndr<M> : Send 
where M: Msg + std::fmt::Debug + Clone + Send + Default,
{
    fn send_message(msg: M, stream: &mut TcpStream) -> Result<()>;
    fn buf_send_message(msg: &dyn Msg, stream: &mut BufWriter<TcpStream>) -> Result<()>;
}
pub trait Rcvr<M> : Send 
where M: Msg + std::fmt::Debug + Clone + Send + Default,
{
    fn recv_message<Que>(stream: &mut TcpStream, q:Que) -> Result<()>;
    fn buf_recv_message<Que>(stream: &mut BufReader<TcpStream>, q:Que) -> Result<()>;
}
pub trait Process<M> : Send 
where M: Msg + std::fmt::Debug + Clone + Send + Default,
{
    fn process_message(&self, m: &dyn Msg) -> &'static dyn Msg;
}
