/////////////////////////////////////////////////////////////
// rust_comm::lib.rs - Tcp Communation Library             //
//   - RustComm_VariableSizeMsg                            //
// Jim Fawcett, https://JimFawcett.github.io, 19 Jul 2020  //
/////////////////////////////////////////////////////////////
/*
   Variable msg size, buffered transfer

   Defined Types:
   - Listener<P,L>
   - Connector<P,M,L>
     - P is a processing type supporting application needs
     - L is a log type which is expected to be either
       VerboseLog or MuteLog
     - M is a message type
   P processes messages and its code must work with that
   of the Message type.
   
   Traits used by these types are defined in rust_traits.
*/

#![allow(unused_imports)]
#![allow(dead_code)]

/*-- rust_comm facilities --*/
use rust_traits::*;
use rust_message::*;
use rust_comm_processing::*;
use rust_blocking_queue::*;
use rust_comm_logger::*;
use rust_thread_pool::*;

/*-- std library facilities --*/
use std::fmt::*;
use std::sync::{Arc, atomic::AtomicBool, atomic::Ordering};
use std::net::{TcpStream, TcpListener, Shutdown};
use std::io::{Result, BufReader, BufWriter, stdout, Write};
use std::io::prelude::*;
use std::thread;
use std::thread::{JoinHandle};

type L = MuteLog;
type M = Message;
type P = CommProcessing<L>;

/*---------------------------------------------------------
  Connector<P,M,L> - attempts to connect to Listener<P,L>
*/
#[derive(Debug)]
pub struct Connector<P,M,L> where 
    M: Msg + Clone + Send + Default,
    P: Debug + Copy + Clone + Send + Sync + Default + Sndr<M> + Rcvr<M>, 
    L: Logger + Debug + Copy + Clone + Default
{
    snd_queue: Arc<BlockingQueue<M>>,
    rcv_queue: Arc<BlockingQueue<M>>,
     _p: P,
     connected: bool,
     log: L,
    //  msg_size: usize,
}
impl<P,M,L> Connector<P,M,L> where
    M: Msg + Clone + Send + Default + 'static,
    P: Debug + Copy + Clone + Send + Sync + Default + Sndr<M> + Rcvr<M>,
    L: Logger + Debug + Copy + Clone + Default
{    
    pub fn is_connected(&self) -> bool {
        self.connected
    }
    pub fn post_message(&self, msg: M) {
        self.snd_queue.en_q(msg);
    }
    pub fn get_message(&self) -> M {
        self.rcv_queue.de_q()
    }
    pub fn has_msg(&self) -> bool {
        self.rcv_queue.len() > 0
    }
    pub fn new(addr: &'static str) -> std::io::Result<Connector<P,M,L>>
    where
        M: Msg + Clone + Send + Default + 'static,
        P: Debug + Copy + Clone + Send + Sync + Default + Sndr<M> + Rcvr<M>,
        L: Logger + Copy + Clone + Default
    {
        let mut _is_connected = false;
        let rslt = TcpStream::connect(addr);
        if rslt.is_err() {
             print!("\n-- connection to {:?} failed --", addr);
             return Err(std::io::Error::new(std::io::ErrorKind::Other, "connect failed"));
        }
        else {
            _is_connected = true;
            L::write(&format!("\n--connected to {:?}--", addr));
        }
        let stream = rslt.unwrap();
        let mut buf_writer = BufWriter::new(stream.try_clone()?);
        let mut buf_reader = BufReader::new(stream.try_clone()?);
        
        let send_queue = Arc::new(BlockingQueue::<M>::new());
        let recv_queue = Arc::new(BlockingQueue::<M>::new());
        
        /*-- send thread reads input queue and sends msg --*/
        let sqm = Arc::clone(&send_queue);
        let _ = std::thread::spawn(move || {
            let ssq = Arc::clone(&sqm);
            loop {
                L::write("\n  -- dequing send msg --");
                let msg = ssq.de_q();
                L::write("\n  sending msg");
                let msg_type = msg.get_type();
                let rslt = P::buf_send_message(&msg, &mut buf_writer);
                if rslt.is_err() {
                    // may cause panic if io doesn't complete before 
                    // thread shuts down
                    // print!("\n  msg send error");
                    break;
                }
                L::write("\n  -- send successful --");
                if msg_type == MessageType::END as u8 {
                    L::write("\n--terminating connector send thread--");
                    break;
                }
            }            
        });
        /*-- recv thread recvs msg (may block) and enQs for user --*/
        let rqm = Arc::clone(&recv_queue);
        let _ = std::thread::spawn(move || {
            let srq = Arc::clone(&rqm);
            loop {
                L::write("\n  attempting to receive msg in connector");
                let rslt = P::buf_recv_message(&mut buf_reader);
                if rslt.is_err() {
                    L::write("\n--terminating connector receive thread--");
                    break;
                }
                let msg = rslt.unwrap();
                srq.en_q(msg);
                L::write(&format!("\n  recv_queue len: {}", srq.len()));
            }
        });
        /*-- return new Connector as std::io::Result --*/
        let me =
        Self {
            _p: P::default(),
            snd_queue: send_queue,
            rcv_queue: recv_queue,
            connected: _is_connected,
            log: L::default(),
            // msg_size: msg_size,
        };
        Ok(me)
    }
}
/*---------------------------------------------------------
  Each threadpool thread executes thread_proc
  - get next TcpStream instance, strm
  - communicate with connecter using handle_client(strm)
*/
pub fn thread_proc(bq: &BlockingQueue<TcpStream>, run: &Arc<AtomicBool>) {
    loop {
        if !run.load(Ordering::Relaxed) {
            print!("\n  terminating listener thread");
            // let _ = std::io::stdout().flush();
            break;
        }
        let strm = bq.de_q();
        let rslt = handle_client(strm);
        if rslt.is_err() {
            print!("\n  stream failure in handle_client");
            break;  // this kills one threadpool thread
        }
    }
}
/*---------------------------------------------------------
  Handle client messages:
  - extract message, msg, from stream 
  - process using reply_msg = P::process_message(msg)
  - send back reply_msg
*/
pub fn handle_client(strm: TcpStream) -> Result<()> {

    /*-- thread handles client until receiving an END or QUIT message --*/
    let mut buf_writer = BufWriter::new(strm.try_clone()?);
    let mut buf_reader = BufReader::new(strm.try_clone()?);
    loop {
        L::write("\n  attempting to recv message in client handler");
        // let _ = std::io::stdout().flush();
        let rslt:Result<Message> = P::buf_recv_message(&mut buf_reader);
        L::write("\n  receive successful in client handler");
        if rslt.is_err() {
            print!("\n  socket session closed abruptly");
            // let _ = std::io::stdout().flush();
            break;
        }
        let mut msg = rslt.unwrap();
        if msg.get_type() == MessageType::END as u8 {
            L::write("\n--listener received END message--");
            L::write("\n--terminating client handler loop--");           
            break;
        }
        else if msg.get_type() == MessageType::QUIT as u8 {
            L::write("\n--listener received QUIT message--");
            L::write("\n--terminating client handler loop--");
            break;
        }
        P::process_message(&mut msg);
        let _ = P::buf_send_message(&msg, &mut buf_writer);
    } 
    L::write("\n  terminating handler thread");
    Ok(())
}
/*---------------------------------------------------------
  Listener<P,L> 
  - attempts to bind to listening address
  - blocks on accept via the incoming iterator
*/
#[derive(Debug)]
pub struct Listener<P,L> 
where 
P: Debug + Copy + Clone + Send + Sync + Default + Sndr<M> + Rcvr<M> + 'static,
L: Logger + Debug + Copy + Clone + Default
{
    p: P,
    run: Arc<AtomicBool>,  // used to terminate Listener
    log: L, 
    num_thrds: u8,
    addr: &'static str,
    // msg_size: usize,
    /*-- ThreadPool instance is aggregated in self.start() --*/
}
impl<P,L> Listener<P,L> 
where 
    P: Debug + Copy + Clone + Send + Sync + Default + Sndr<M> + Rcvr<M> + Process<M> + 'static,
    L: Logger + Debug + Copy + Clone + Default
    {    
    pub fn new(nt: u8) -> Listener<P,L> {
        Listener {
              p: P::default(),
              run: Arc::new(AtomicBool::new(true)),
              log: L::default(),
              num_thrds: nt,
              addr: "",
            //   msg_size: 64,
        }
    }
    // pub fn set_msg_size(&mut self, msg_size:usize) {
    //     self.msg_size = msg_size;
    // }
    // pub fn get_msg_size(&self) -> usize {
    //     self.msg_size
    // }
    /*-- starts thread wrapping incoming loop which often blocks --*/
    pub fn start(&mut self, addr: &'static str) -> Result<JoinHandle<()>> 
    {
        self.addr = addr;
        L::write(&format!("\n--starting listener on {:?}--", addr));
        let rslt = TcpListener::bind(addr);
        if rslt.is_err() {
            print!("\n  binding to {:?} failed", addr);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "listener bind failed"));
        }
        let tcpl = rslt.unwrap();
        let nt = self.num_thrds;
        let run_ref = Arc::clone(&self.run);

        /*-- this outer thread prevents appl from blocking waiting for connections --*/
        let handle = std::thread::spawn(move || {
            let mut tp = ThreadPool::<TcpStream>::new(nt, thread_proc);
            /*-- loop on incoming iterator which calls accept and so blocks --*/
            for stream in tcpl.incoming() {
                if !run_ref.load(Ordering::Relaxed) {
                    break;
                }
                if stream.is_ok() {
                    tp.post(stream.unwrap());
                }
                else {
                    continue;
                }
            }
            tp.stop();
            L::write("\n--terminating listener thread--");  
        });
        Ok(handle)
    }
    pub fn stop(&mut self) {
        self.run.store(false, Ordering::Relaxed);
        let conn = Connector::<P,M,L>::new(self.addr).unwrap();
        let mut msg = Message::new(TYPE_SIZE + CONTENT_SIZE + 1);
        msg.set_type(MessageType::QUIT as u8);
        conn.post_message(msg);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
