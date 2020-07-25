/////////////////////////////////////////////////////////////
// rust_comm::test3.rs - Test Tcp Communation Library      //
//                                                         //
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
use std::sync::Arc;

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
        name, num_msgs, sz_bytes + 1
    );
    let conn = Connector::<P,M,Log>::new(addr).unwrap();
    let mut msg = Message::new();
    let body: Vec<u8> = vec![0u8;sz_bytes];
    msg.set_body_bytes(body);
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
        let mut msg = Message::new();
        msg.set_type(MessageType::END);
        conn.post_message(msg);
        display_test_data(et, num_msgs, sz_bytes);
    });
    handle
}
fn display_test_data(et:u128, num_msgs:usize, msg_size:usize) {
    let elapsed_time_sec = 1.0e-6 * et as f64;
    let num_msgs_f64 = num_msgs as f64;
    let size_mb = 1.0e-6*(msg_size + 1) as f64;
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
) -> std::thread::JoinHandle<()> 
{
    print!(
        "\n  -- {}: {} msgs, {} bytes per msg ", 
        name, num_msgs, sz_bytes + 1
    );
    let conn = Arc::new(Connector::<P,M,Log>::new(addr).unwrap());
    let sconn1 = Arc::clone(&conn);
    let sconn2 = Arc::clone(&conn);
    let mut msg = Message::new();
    let body: Vec<u8> = vec![0u8;sz_bytes];
    msg.set_body_bytes(body);
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
        let mut msg = Message::new();
        msg.set_type(MessageType::END);
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
  handle
}

/*---------------------------------------------------------
  Perf testing - runs tests of the day
*/
fn main() {

    type L = MuteLog;

    print!("\n  -- test3: rust_comm --\n");
    
    let addr = "127.0.0.1:8080";
    let mut lsnr = Listener::<P,Log>::new();
    let rslt = lsnr.start(addr);
    if rslt.is_err() {
        return;
    }
    let handle = rslt.unwrap();

    let h1 = client_wait_for_reply::<L>(
        addr, "test3 - wait for reply", 1000, 65536
    );
    let _ = h1.join();
    println!();

    let h2 = client_no_wait_for_reply::<L>(
        addr, "test3 - no wait for reply", 1000, 65536
    );
    let _ = h2.join();
    println!();

    let h1 = client_wait_for_reply::<L>(
        addr, "test3 - wait for reply", 1000, 1024
    );
    let _ = h1.join();
    println!();

    let h2 = client_no_wait_for_reply::<L>(
        addr, "test3 - no wait for reply", 1000, 1024
    );
    let _ = h2.join();
    println!();

    let h1 = client_wait_for_reply::<L>(
        addr, "test3 - wait for reply", 1000, 0
    );
    let _ = h1.join();
    println!();

    let h2 = client_no_wait_for_reply::<L>(
        addr, "test3 - no wait for reply", 1000, 0
    );
    let _ = h2.join();
    println!();

    /*-- shut down listener --*/
    let conn = Connector::<P,M,Log>::new(addr).unwrap();
    let mut msg = Message::new();
    msg.set_type(MessageType::QUIT);
    L::write(
        &format!("\n  main posting  {:?} msg", "QUIT")
    );
    conn.post_message(msg);
    let _ = std::io::stdout().flush();

    let _ = handle.join();
    println!();
}