/////////////////////////////////////////////////////////////
// rust_comm::test4.rs - Test Tcp Communication Library    //
//   - RustComm_VariableSizeMsg_NoBuff                     //
// Jim Fawcett, https://JimFawcett.github.io, 19 Jul 2020  //
/////////////////////////////////////////////////////////////
/*
   Fixed size msg: buffered, fixed size msgs

   NOTE!  message size is defined in Message crate
 
   Demo:
   Test message rate and throughput for multiple clients
   - start Listener component
   - start nc Connector components, each on its own thread
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
use std::thread::{JoinHandle};

/*-- component library rust_blocking_queue --*/
use rust_message::*;
use rust_traits::*;
use rust_comm_processing::*;
use rust_comm_logger::*;
use rust_comm::*;
use rust_timer::*;

type Log = MuteLog;
type M = Message;
type P = CommProcessing<Log>;

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
        "\n  -- {}: {} msgs, {} bytes per msg ", 
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
            let _msg = conn.get_message();
            L::write(
                &format!(
                    "\n  received msg: {:?}", 
                    &_msg.type_display()
                )
            );
        }
        let _ = tmr.stop();
        let et = tmr.elapsed_micros();
        let content_size = 32;
        let mut msg = Message::new(HEADER_SIZE + content_size);
        msg.set_type(MessageType::END as u8);
        msg.set_content_size(content_size);
        conn.post_message(msg);
        display_test_data(et, num_msgs, sz_bytes);
        let _ = std::io::stdout().flush();
    });
    handle
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
        "\n  -- {}: {} msgs, {} content bytes per msg ", 
        name, num_msgs, sz_bytes
    );
    let conn = Arc::new(Connector::<P,M,Log>::new(addr).unwrap());
    let sconn1 = Arc::clone(&conn);
    let sconn2 = Arc::clone(&conn);
    let msg = Message::create_msg_bytes_fit(&vec![0;sz_bytes]);
    let _handle = std::thread::Builder::new().name("first".to_string()).spawn(move || {
        for _i in 0..num_msgs {
            L::write(
                &format!(
                    "\n  posting msg   {:?} of size:  {:?}", 
                    name, sz_bytes
                )
            );
            sconn1.post_message(msg.clone());
        }
        let content_size = 32;
        let mut msg = Message::new(HEADER_SIZE + content_size);
        msg.set_type(MessageType::END as u8);
        msg.set_content_size(content_size);
        sconn1.post_message(msg);
    });
    let handle = std::thread::Builder::new().name("second".to_string()).spawn(move || {
        for _i in 0..num_msgs {
            let msg = sconn2.get_message();
            L::write(
                &format!(
                    "\n  received msg: {:?}", 
                    &msg.type_display()
                )
            );
        }
    });
  (_handle.unwrap(), handle.unwrap())
}
/*---------------------------------------------------------
  Display test data - used for individual tests
*/
fn display_test_data(et:u128, num_msgs:usize, msg_size:usize) {
    let elapsed_time_sec = 1.0e-6 * et as f64;
    let num_msgs_f64 = num_msgs as f64;
    let size_mb = 1.0e-6*(msg_size) as f64;
    let msg_rate = num_msgs_f64/elapsed_time_sec;
    let byte_rate_mbpsec = num_msgs_f64*size_mb/elapsed_time_sec;
    print!("\n      elapsed microsec {}", et);
    print!("\n      messages/second  {:.2}", msg_rate);
    print!("\n      thruput - MB/S   {:.2}", byte_rate_mbpsec);
    let _ = std::io::stdout().flush();
}
/*---------------------------------------------------------
  Multiple clients running client_no_wait_for_reply
*/
fn multiple_clients(
    nc: u8,
    addr: &'static str,     // endpoint: Ipaddr:Port
    name: &'static str,     // test name
    num_msgs:usize,         // number of messages
    sz_bytes:usize          // message body size
)
{
    print!("\n  number of clients:  {:?}",nc);
    let mut tmr = StopWatch::new();
    tmr.start();
    let mut handles = Vec::<Option<JoinHandle<()>>>::new();
    for _i in 0..nc {
        let (h2a, h2b) = client_no_wait_for_reply::<MuteLog>(addr, name, num_msgs, 4096);
        handles.push(Some(h2a));
        handles.push(Some(h2b));
    }
    /*-- wait for all replies --*/
    for handle in &mut handles {
        let _ = handle.take().unwrap().join();
    }
    tmr.stop();
    /*-----------------------------------------------------
      Note: scaling microseconds to seconds cancels 
            scaling bytes to Megabytes
    */
    let et = tmr.elapsed_micros();  // divided by 10e6 to get sec
    let nm = nc as usize *num_msgs;
    let tp = (nm * sz_bytes) as u128 / et; // divided by 10e6 to get MB
    print!("\n  elapsed microsecs:  {:?}",et);
    print!("\n  number messages:    {:?}", nm);
    print!("\n  throughput MB/S:    {:?}", tp)
}
/*---------------------------------------------------------
  Perf testing - runs tests of the day
*/

fn main() {

    print!("\n  -- Demo rust_comm: test3\n  -- VariableMsgSize, Buffered\n");

    type L = MuteLog;
    
    let nt: u8 = 8;
    let addr = "127.0.0.1:8080";
    print!("\n  num thrdpool thrds: {:?}",nt);
    let mut lsnr = Listener::<P,Log>::new(nt);
    let rslt = lsnr.start(addr);
    if rslt.is_err() {
        return;
    }
    let _handle = rslt.unwrap();

    multiple_clients(16, addr, "test4", 1000, 4096);
    println!();

    /*-- shut down listener --*/
    lsnr.stop();
    let _ = _handle.join();
}