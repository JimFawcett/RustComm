/////////////////////////////////////////////////////////////
// rust_comm_processing::test2.rs - test send/recv         //
//                                                         //
// Jim Fawcett, https://JimFawcett.github.io, 12 Sep 2020  //
/////////////////////////////////////////////////////////////

#![allow(unused_imports)]
use std::net::{TcpListener, TcpStream};

use rust_traits::*;
use rust_comm_processing::*;
use rust_message::*;
use rust_comm_logger::*;
use std::sync::*;
use std::io::*;

type Log = MuteLog;

fn handle_client(stream: &TcpStream) -> std::io::Result<()> {
    let mut buf_writer = BufWriter::new(stream.try_clone()?);
    let mut buf_reader = BufReader::new(stream.try_clone()?);

    let rslt:Result<Message> = CommProcessing::<Log>::buf_recv_message(&mut buf_reader);
    if rslt.is_err() {
        print!("\n  recv_message error");
        let err = std::io::Error::new(ErrorKind::Other, "recv error");
        return Err(err);
    }
    else {
        print!("\n  receiver received msg");
        let msg = rslt.unwrap();
        msg.show_message(8);
        CommProcessing::<Log>::buf_send_message(&msg, &mut buf_writer)?;
    }
    Ok(())
}
fn start_listener(end_point: &str) -> std::io::Result<()> {
    let tcpl = TcpListener::bind(end_point)?;
    for stream in tcpl.incoming() {
        print!("\n  listener accepted connection");
        let rslt = handle_client(&stream?);
        if rslt.is_err() {
            print!("\n  error in handle_client");
            let _ = std::io::stdout().flush();
        }
        break;  // only one connection for testing
    }
    Ok(())
}
fn construction(addr: &'static str) -> Result<()> {
    let _cp = CommProcessing::<VerboseLog>::new();
    let addr_copy = addr;
    let handle = std::thread::spawn(move || {
        let rslt = start_listener(addr_copy);
        if rslt.is_err() {
            print!("\n  failed to start listener on {:?}",addr);
            let _ = std::io::stdout().flush();
        }
        rslt
    });

    Log::write("\n  sending msg");
    let mut msg:Message = Message::create_msg_str_fit("test message");
    msg.set_type(MessageType::FLUSH as u8);
    let stream = TcpStream::connect(addr)?;
    let mut buf_writer = BufWriter::new(stream.try_clone()?);
    let mut buf_reader = BufReader::new(stream.try_clone()?);
    CommProcessing::<Log>::buf_send_message(&msg, &mut buf_writer)?;
    Log::write(&format!("\n  sent message with len: {:?}",msg.len()));
    let _ = std::io::stdout().flush();
    println!();

    let mut msg:Message = CommProcessing::<Log>::buf_recv_message(&mut buf_reader)?;
    print!("\n\n  connector received reply msg");
    let _ = std::io::stdout().flush();

    msg.show_message(8);
    let s = msg.get_content_str().unwrap();
    print!("\n\n  message content: {:?}",s);

    msg.set_type(MessageType::QUIT as u8);
    CommProcessing::<Log>::buf_send_message(&msg, &mut buf_writer)?;
    let msg:Message = CommProcessing::<Log>::buf_recv_message(&mut buf_reader)?;
    msg.show_message(8);
    let _ = handle.join();
    Ok(())
}

fn main() {

    print!("\n  -- test2 : VariableSizeMsg\n");

    let addr: &'static str = "127.0.0.1:8083";
    let rslt = construction(addr);
    if rslt.is_err() {
        print!("\n  listener start on {:?} failed", addr);
    }

    print!("\n  That's all Folks!\n\n");
}