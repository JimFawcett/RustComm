/////////////////////////////////////////////////////////////
// rust_comm_processing::test1.rs - test send/recv         //
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
    let mut clone_stream = stream.try_clone()?;
    let rslt = CommProcessing::<Log>::recv_message(&mut clone_stream);
    if rslt.is_err() {
        print!("\n  recv_message error");
        let err = std::io::Error::new(ErrorKind::Other, "recv error");
        return Err(err);
    }
    else {
        let msg:Message = rslt.unwrap();
        print!("\n  receiver received msg");
        msg.show_message(8);
        CommProcessing::<Log>::send_message(&msg, &mut clone_stream)?;
    }
    Ok(())
}
fn start_listener(end_point: &str) -> std::io::Result<()> {
    let tcpl = TcpListener::bind(end_point)?;
    for stream in tcpl.incoming() {
        print!("\n  listener accepted connection");
        handle_client(&stream?)?;
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
    let msg = Message::create_msg_str_fit("test string");
    let mut stream = TcpStream::connect(addr)?;
    let mut clone_stream = stream.try_clone()?;
    let _ = CommProcessing::<Log>::send_message(&msg, &mut stream);

    let msg:Message = CommProcessing::<Log>::recv_message(&mut clone_stream)?;
    println!();

    print!("\n  connector received reply msg");
    msg.show_message(8);
    let s = msg.get_content_str().unwrap();
    print!("\n\n  message content: {:?}",s);
    let _ = handle.join();
    Ok(())
}

fn main() {

    print!("\n  -- test1 : VariableSizeMsg\n");

    // let addr: &'static str = "127.0.0.1:0";  // test listen failure
    let addr: &'static str = "127.0.0.1:8083";
    let rslt = construction(addr);
    if rslt.is_err() {
        print!("\n  listener start on {:?} failed", addr);
        let _ = std::io::stdout().flush();
    }

    print!("\n  That's all Folks!\n\n");
}