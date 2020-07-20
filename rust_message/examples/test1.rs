/////////////////////////////////////////////////////////////
// rust_message::test1.rs - demonstrate message type       //
//                                                         //
// Jim Fawcett, https://JimFawcett.github.io, 20 Jul 2020  //
/////////////////////////////////////////////////////////////

use rust_traits::*;
use rust_message::*;

fn main() {
    let mut msg = Message::new();
    msg.set_type(MessageType::TEXT);
    let contents = String::from("a test string");
    msg.set_body_str(&contents);
    //let sz = msg.get_body_size();
    msg.show_msg();

  print!("\n\n  That's all Folks!\n\n");
}