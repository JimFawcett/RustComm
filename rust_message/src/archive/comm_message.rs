// comm_message

#![allow(dead_code)]

use std::fmt::*;
use crate::comm_traits::*;

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
    pub const REPLY:u8 = 16;

    pub fn _get_type(&self) -> u8 {
        self.msgtype
    }
    pub fn _set_type(&mut self, mt:u8) {
        if (mt == MessageType::TEXT) | 
        (mt == MessageType::BYTES) | 
        (mt == MessageType::END) |
        (mt == MessageType::QUIT) |
        (mt == MessageType::REPLY) {
            self.msgtype = mt;
        }
    }
}

#[derive(Debug, Clone)]
pub struct Message {
    mt: u8,
    body_buffer: Vec<u8>,
}
impl Message {
    pub fn new() -> Message {
        Message {
            mt: 1,
            body_buffer: Vec::<u8>::new(),
        }
    }
    pub fn bytes_to_vec(ba: &[u8]) -> Vec<u8> {
        let v:Vec<u8> = ba.iter().cloned().collect();
        v
    }
    
    pub fn show_msg(msg: &Message) {
        print!("\n  {}", msg);
    }
    
    pub fn show_body(msg: &Message) {
        print!("\n  {:?}", msg.get_body_bytes());
    }
    
    pub fn show_body_str(msg: &Message) {
        let s = msg.get_body_str();
        print!("\n  {:?}", s);
    }
}
impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "{{\n    MessageType: {:?}\n    body size: {}\n  }}", 
            self.type_display(), self.body_buffer.len()
        )
    } 
}
impl Default for Message {
    fn default() -> Message {
        Message {
            mt: 1,
            body_buffer: Vec::<u8>::new(),
        }
    }
}
impl Msg for Message {
    // fn new() -> &'static dyn Msg {
    //     Message {
    //         mt: MessageType { msgtype: 1, },
    //         body_buffer: Vec::<u8>::new(),
    //     }
    // }
    fn set_type(&mut self, mt: u8) {
        self.mt = mt;
    }
    fn get_type(&self) -> u8 {
        self.mt
    }
    fn set_body_bytes(&mut self, b:Vec<u8>) {
        self.body_buffer = b;
    }
    fn set_body_str(&mut self, s: &str) {
        let buff = s.as_bytes();
        for byte in buff {
            self.body_buffer.push(*byte);
        }
    }
    fn get_body_size(&self) -> usize {
        self.body_buffer.len()
    }
    fn get_body_bytes(&self) -> &Vec<u8> {
        &self.body_buffer
    }
    fn get_body_str(&self) -> String {
        let s = String::from_utf8_lossy(&self.body_buffer);
        s.to_string()
    }
    fn clear(&mut self) {
        self.body_buffer.clear();
    }
    fn type_display(&self) -> String {
        let typ:u8 = self.mt;
        let mut s = String::new();
        if typ == MessageType::TEXT {
            s.push_str("TEXT");
        }
        else if typ == MessageType::BYTES {
            s.push_str("BYTES");
        }
        else if typ == MessageType::BYTES {
            s.push_str("END");
        }
        else if typ == MessageType::BYTES {
            s.push_str("QUIT");
        }
        else {
            s.push_str("REPLY");
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
    print!("\n  {:?}", msg.get_body_bytes());
}

pub fn show_body_str(msg: &Message) {
    let s = msg.get_body_str();
    print!("\n  {:?}", s);
}
