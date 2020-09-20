/////////////////////////////////////////////////////////////
// rust_comm::test3.rs - Test Tcp Communation Library      //
//   - RustComm_VariableSizeMsg_NoBuff                     //
// Jim Fawcett, https://JimFawcett.github.io, 19 Jul 2020  //
/////////////////////////////////////////////////////////////
/*
   Demo:
   Test message rate and throughput
   - start Listener component
   - start Connector component
   - start post_message thread
   - start recv_message thread
   - send a fixed number of messages
   - send END message to exit client handler
   - eval elapsed time
   - send QUIT message to shut down Listener
*/
#![allow(unused_imports)]
#![allow(dead_code)]

use std::io::prelude::*;
use std::net::{Shutdown, TcpStream};
use std::sync::Arc;
use std::{thread, time};

/*-- component library rust_blocking_queue --*/
use rust_message::*;
use rust_traits::*;
use rust_comm_processing::*;
use rust_comm_logger::*;
use rust_comm::*;
use rust_timer::*;
use rust_debug::*;

type Log = MuteLog;
type M = Message;
type P = CommProcessing<Log>;

// fn flush_out() {
//     let _ = std::io::stdout().flush();
// }
// fn break_here<F>(pred:bool, f:F) where F:Fn() {
//     if pred {
//         print!("\n  break: press return to continue");
//         flush_out();
//         f();
//         let mut _buff = String::new();
//         let _ = std::io::stdin().read_line(&mut _buff);
//     }
// }
/*---------------------------------------------------------
  Perf test - client waits for reply before posting again
*/
fn client_wait_for_reply<L: Logger>(
    addr: &'static str,     // endpoint address Ipaddr:Port
    name: &'static str,     // test name
    num_msgs:usize,         // number of messages to send
    sz_bytes:usize          // body size in bytes
) -> std::thread::JoinHandle<()> 
{
    print!(
        "\n  -- {}:\n      {} msgs, {} bytes content per msg ", 
        name, num_msgs, sz_bytes
    );
    let conn = Connector::<P,M,Log>::new(addr).unwrap();
    let mut msg = Message::create_msg_bytes_fit(&vec![0;sz_bytes]);
    msg.set_type(MessageType::FLUSH as u8);

    let mut tmr = StopWatch::new();
    let handle = std::thread::spawn(move || {
        /*-- start timer after connect, bld msg & start thread --*/
        tmr.start();
        for _i in 0..num_msgs {
            L::write(
                &format!(
                    "\n  posting msg   {:?} of size:  {:?}", 
                    name, sz_bytes
                )
            );
            conn.post_message(msg.clone());
            let msg = conn.get_message();
            L::write(
                &format!(
                    "\n  received msg: {:?}", 
                    &msg.type_display()
                )
            );
        }
        let _ = tmr.stop();
        let et = tmr.elapsed_micros();
        let mut msg = Message::create_msg_header_only();
        msg.set_type(MessageType::END as u8);
        conn.post_message(msg);
        display_test_data(et, num_msgs, sz_bytes);
    });
    handle
}
fn display_test_data(et:u128, num_msgs:usize, msg_size:usize) {
    let elapsed_time_sec = 1.0e-6 * et as f64;
    let num_msgs_f64 = num_msgs as f64;
    let size_mb = 1.0e-6*(msg_size) as f64;
    let msg_rate = num_msgs_f64/elapsed_time_sec;
    let byte_rate_mbpsec = num_msgs_f64*size_mb/elapsed_time_sec;
    print!("\n      elapsed microsec {}", et);
    print!("\n      messages/second  {:.2}", msg_rate);
    print!("\n      thruput - MB/S   {:.2}", byte_rate_mbpsec);
}

/*---------------------------------------------------------
  Perf test - client does not wait for reply before posting
*/
fn client_no_wait_for_reply<L: Logger>(
    addr: &'static str,     // endpoint: Ipaddr:Port
    name: &'static str,     // test name
    num_msgs:usize,         // number of messages
    sz_bytes:usize          // message body size
) -> (std::thread::JoinHandle<()>, std::thread::JoinHandle<()>) 
{
    print!(
        "\n  -- {}:\n      {} msgs, {} bytes content per msg ", 
        name, num_msgs, sz_bytes
    );
    let conn = Arc::new(Connector::<P,M,Log>::new(addr).unwrap());
    let sconn1 = Arc::clone(&conn);
    let sconn2 = Arc::clone(&conn);
    let msg = Message::create_msg_bytes_fit(&vec![0;sz_bytes]);

    let mut tmr = StopWatch::new();
    let _handle = std::thread::spawn(move || {
        /*-- start timer after connect, bld msg & start thread --*/
        tmr.start();
        for _i in 0..num_msgs {
            L::write(
                &format!(
                    "\n  posting msg   {:?} of size:  {:?}", 
                    name, sz_bytes
                )
            );
            sconn1.post_message(msg.clone());
        }
        let mut msg = Message::new(TYPE_SIZE + CONTENT_SIZE);
        msg.set_type(MessageType::END as u8);
        sconn1.post_message(msg);
    });
    let handle = std::thread::spawn(move || {
        for _i in 0..num_msgs {
            let msg = sconn2.get_message();
            L::write(
                &format!(
                    "\n  received msg: {:?}", 
                    &msg.type_display()
                )
            );
        }
        /*-- stop timer after receiving last message --*/
        let _ = tmr.stop();
        let et = tmr.elapsed_micros();
        display_test_data(et, num_msgs, sz_bytes);
    });
  (_handle, handle)
}

/*---------------------------------------------------------
  Perf testing - runs tests of the day
*/
fn main() {

    print!("\n  -- Demo rust_comm: test3");
    print!("\n  -- One client");
    print!("\n  -- VariableMsgSize, Buffered\n");

    type L = MuteLog;

    let nt: u8 = 8;
    let addr = "127.0.0.1:8080";
    let mut lsnr = Listener::<P,Log>::new(nt);
    let rslt = lsnr.start(addr);
    if rslt.is_err() {
        return;
    }
    let _handle = rslt.unwrap();

    let h1 = client_wait_for_reply::<L>(
        addr, "test3 - wait for reply", 1000, 65536
    );
    let _ = h1.join();
    println!();

    let (h2a, h2b) = client_no_wait_for_reply::<L>(
        addr, "test3 - no wait for reply", 1000, 65536
    );
    let _ = h2a.join();
    let _ = h2b.join();
    println!();

    let h1 = client_wait_for_reply::<L>(
        addr, "test3 - wait for reply", 1000, 1024
    );
    let _ = h1.join();
    println!();

    let (h2a, h2b) = client_no_wait_for_reply::<L>(
        addr, "test3 - no wait for reply", 1000, 1024
    );
    let _ = h2a.join();
    let _ = h2b.join();
    println!();

    let h1 = client_wait_for_reply::<L>(
        addr, "test3 - wait for reply", 1000, 0
    );
    let _ = h1.join();
    println!();

    let (h2a, h2b) = client_no_wait_for_reply::<L>(
        addr, "test3 - no wait for reply", 1000, 0
    );
    let _ = h2a.join();
    let _ = h2b.join();

    println!();
    
    /*-- shut down listener --*/
    lsnr.stop();
    let _ = _handle.join();
}