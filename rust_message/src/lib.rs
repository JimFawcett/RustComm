/////////////////////////////////////////////////////////////
// rust_message - message type used to test rust_comm      //
//                                                         //
// Jim Fawcett, https://JimFawcett.github.io, 20 Jul 2020  //
/////////////////////////////////////////////////////////////
/*
   Message:
   - fixed size header holding a MessageType attribute:
     TEXT, BYTES, END, QUIT, REPLY
   - body holds utf-8 text or arbitrary byte sequence
   - stores contents in std::Vec<u8>
*/

#![allow(dead_code)]

// use std::fmt::*;
use rust_traits::*;
use std::str::Utf8Error;

/*---------------------------------------------------------
  Message:
  - structure that wraps Vec<u8>, treated as byte array
  - set of public functions for manipulating Message state
*/
pub const TYPE_SIZE:usize = 1;
pub const CONTENT_SIZE:usize = 8;  // max 4096 - 32 - 1 = 4063
pub const HEADER_SIZE:usize = TYPE_SIZE + CONTENT_SIZE;

#[repr(u8)]
pub enum MessageType {
    DEFAULT = 0,
    TEXT = 1,
    REPLY = 2,
    END = 4,
    QUIT = 8,
    FLUSH = 16,
}

 #[derive(Debug, Clone, Default)]
pub struct Message {
    br: Vec<u8>,
} 
impl Msg for Message {
    /*-------------------------------------------
      Primary interface
    */
    fn new(sz:usize) -> Self {
        assert!(sz >= HEADER_SIZE);
        Self {
            br: vec![0; sz],
        }
    }
    /*-- load existing heap array with zeros --*/
    fn init(&mut self) {
        let sz = self.len();
        self.br = vec![0;sz];
    }
    /*-- return message length --*/
    fn len(&self) -> usize {
        self.br.len()
    }
    fn is_empty(&self) -> bool {
        self.br.len() == 0
    }
    /*-- set message MsgType --*/
    fn set_type(&mut self, mt:u8) {
        self.br[0] = mt as u8;
    }
    fn get_type(&self) -> u8 {
        self.br[0]
    }
    /*-------------------------------------------
      Set message content from buff and set
      content size to length of buff
    */
    fn set_content_bytes(&mut self, buff: &[u8]) {
        self.set_content_size(buff.len());
        self.set_field(HEADER_SIZE, buff);
    }
    fn get_content_bytes(&self) -> &[u8] {
        self.get_field(
            HEADER_SIZE, 
            self.get_content_size()
        )
    }
    /*-------------------------------------------
      Set message content from str and set
      content size to length of str
    */
    fn set_content_str(&mut self, s:&str) {
        self.set_content_size(s.len());
        self.set_content_bytes(s.as_bytes());
    }
    fn get_content_str(&self) ->Result<&str, Utf8Error> {
        let sz = self.get_content_size();
        let start = HEADER_SIZE;
        let end = start + sz;
        Self::str_from_bytes(&self.br[start..end])
    }
    /*-- set message content size --*/
    fn set_content_size(&mut self, sz:usize) {
        self.set_field(TYPE_SIZE, &sz.to_be_bytes());
    }
    fn get_content_size(&self) -> usize {
        let bytes = self.get_field(TYPE_SIZE, CONTENT_SIZE);
        let mut dst = [0u8;8];
        dst.clone_from_slice(bytes); // array from byte slice
        usize::from_be_bytes(dst)    // usize from byte array
    }
    fn get_bytes(&self) -> &[u8] {
        &self.br[..]
    }
    fn get_mut_bytes(&mut self) -> &mut [u8] {
        &mut self.br[..]
    }
    fn set_bytes(&mut self, buff:&[u8]) {
        for i in 0..buff.len() {
            self.br[i] = buff[i];
        }
    }
    fn get_ref(&self) -> &Vec<u8> {
        &self.br
    }
    fn get_mut_ref(&mut self) -> &mut Vec<u8> {
        &mut self.br
    }
    /*-------------------------------------------
      Display message with folded contents
    */
    fn show_message(&self, fold:usize) {
        let mut foldpoint = 0;
        loop {
            print!("\n  ");
            for i in 0..fold {
                if i + foldpoint < self.br.len() {
                    print!("{:>3} ", self.br[i + foldpoint]);
                }
                else {
                    return;
                }
            }
            foldpoint += fold;
        }
    }
    fn type_display(&self) -> String {
        let mut rtn:String = String::from("UNKNOWN");
        if self.br[0] == MessageType::DEFAULT as u8 {
            rtn = String::from("DEFAULT");
        }
        else if self.br[0] == MessageType::END as u8 {
            rtn = String::from("END");
        }
        else if self.br[0] == MessageType::QUIT as u8 {
            rtn = String::from("QUIT");
        }
        else if self.br[0] == MessageType::REPLY as u8 {
            rtn = String::from("REPLY");
        }
        else if self.br[0] == MessageType::TEXT as u8 {
            rtn = String::from("TEXT");
        }
        else if self.br[0] == MessageType::FLUSH as u8 {
            rtn = String::from("FLUSH");
        }
        rtn
    }
}
impl Message {
    /*-------------------------------------------
      Secondary interface
    */
    pub fn create_msg_str_fit(content: &str) -> Message {
        let msg_size = content.len() + HEADER_SIZE;
        let mut msg = Message::new(msg_size);
        let cnt_len = content.len();
        msg.set_content_size(cnt_len);
        if cnt_len > 0 {
            msg.set_content_str(content);
        }
        msg
    }
    pub fn create_msg_bytes_fit(content: &[u8]) -> Message {
        let msg_size = content.len() + HEADER_SIZE;
        let mut msg = Message::new(msg_size);
        let cnt_len = content.len();
        msg.set_content_size(cnt_len);
        if cnt_len > 0 {
            msg.set_content_bytes(content);
        }
        msg
    }
    pub fn create_msg_header_only() -> Message {
        let mut msg = Message::new(HEADER_SIZE);
        msg.set_content_size(0);
        msg
    }
    pub fn set_field(&mut self, offset:usize, buff: &[u8]) {
        for (i, item) in buff.iter().enumerate() {
            if i + offset < self.br.len() {
                self.br[i + offset] = *item;
            }
        }
    }
    pub fn get_field(&self, offset:usize, size:usize) -> &[u8] {
        &self.br[offset..offset+size]
    }
    pub fn set_str(&mut self, offset:usize, s:&str) {
        let buff = Self::str_to_bytes(s);
        self.set_field(offset, buff);
    }
    pub fn get_str(&self, offset:usize, size:usize) 
        -> Result<&str, Utf8Error> {
        Self::str_from_bytes(&self.br[offset..offset+size])
    }
    pub fn str_to_bytes(s:&str) -> &[u8] {
        s.as_bytes()
    }
    pub fn str_from_bytes(b: &[u8]) -> Result<&str, Utf8Error> {
        std::str::from_utf8(b)
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    #[test]
    fn construction() {
        // let mut msg = Message::new();
        // msg.set_type(MessageType::TEXT);
        // let contents = String::from("a test string");
        // msg.set_body_str(&contents);
        // let sz = msg.get_body_size();
        // assert_eq!(sz, MSGSIZE);
    }
}
