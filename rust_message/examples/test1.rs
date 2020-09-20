/////////////////////////////////////////////////////////////
// rust_message::test1.rs - demonstrate message type       //
//                                                         //
// Jim Fawcett, https://JimFawcett.github.io, 20 Jul 2020  //
/////////////////////////////////////////////////////////////

use rust_traits::*;
use rust_message::*;
// use rust_message::{MessageType};

const MESS_SIZE:usize = 32;

fn main() {
    print!("\n  -- demo Message type --\n");

    print!("\n  -- demo writing directly to msg buffer --\n");
    let mut msg = Message::new(MESS_SIZE);
    print!("\n  msg len: {:?}",msg.get_ref().len());
    for i in TYPE_SIZE + CONTENT_SIZE..MESS_SIZE {
      msg.get_mut_ref()[i] = i as u8;
    }
    msg.set_content_size(MESS_SIZE);
    msg.show_message(8);
    println!();

    print!("\n  -- demo load/unload str --\n");
    msg.set_type(MessageType::TEXT as u8);
    let contents = String::from("a test string");
    msg.set_content_str(&contents);
    print!("\n  contents: {:?}",contents);
    msg.show_message(8);
    let sz = msg.get_content_size();
    print!("\n  content size: {:?}",sz);
    let rslt = msg.get_content_str();
    if let Ok(_) = rslt {
      print!("\n  contents: {:?}",rslt.unwrap());
    }
    println!();

    print!("\n  -- demo load/unload byte buffer --\n");
    msg.init();
    let bytes = [1,2,3,4];
    msg.set_content_bytes(&bytes);
    print!("\n  bytes: {:?}",bytes);
    msg.show_message(8);
    let sz = msg.get_content_size();
    print!("\n  content size: {:?}",sz);
    let bytes = msg.get_content_bytes();
    print!("\n  bytes: {:?}",bytes);
    println!();

    print!("\n  -- demo setting MessageType --\n");
    msg.set_type(MessageType::TEXT as u8);
    msg.show_message(8);
    let mt = msg.get_type();
    print!("\n  mt: {}, MessageType: {:?}",mt, msg.type_display());
    println!();

    print!("\n  -- demo messages fitted to content --\n");
    let msg = Message::create_msg_str_fit("a test string");
    msg.show_message(8);
    print!("\n\n  content: {:?}",msg.get_content_str().unwrap());
    println!();

    let msg = Message::create_msg_bytes_fit(&[1, 2, 3, 4]);
    msg.show_message(8);
    print!("\n\n  content: {:?}",msg.get_content_bytes());
    println!();

    let msg = Message::create_msg_bytes_fit(&[0u8;0]);  // intentionally 0 length
    msg.show_message(8);
    print!("\n\n  content: {:?}",msg.get_content_bytes());
    let sz = msg.get_content_size();
    print!("\n\n  msg content size: {}",sz);
    println!();

    print!("\n  -- demo header only message --\n");
    let msg = Message::create_msg_header_only();
    let sz = msg.get_content_size();
    msg.show_message(8);
    print!("\n\n  msg content size: {}",sz);
    
    print!("\n\n  That's all Folks!\n\n");
}