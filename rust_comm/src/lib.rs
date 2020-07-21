/////////////////////////////////////////////////////////////
// rust_comm::lib.rs - Tcp Communation Library             //
//                                                         //
// Jim Fawcett, https://JimFawcett.github.io, 19 Jul 2020  //
/////////////////////////////////////////////////////////////
/*
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

/*-- std library facilities --*/
use std::fmt::*;
use std::sync::{Arc, atomic::AtomicBool, atomic::Ordering};
use std::net::{TcpStream, TcpListener, Shutdown};
use std::io::{Result, BufReader, BufWriter, stdout, Write};
use std::io::prelude::*;
use std::thread;
use std::thread::{JoinHandle};

pub type M = Message;
pub type P<L> = CommProcessing<L>;

/*---------------------------------------------------------
  Connector<P,M,L> - attempts to connect to Listener<P,L>
*/
#[derive(Debug)]
pub struct Connector<P,M,L> where 
    M: Msg + Debug + Clone + Send + Default,
    P: Debug + Copy + Clone + Send + Sync + Default + Sndr<M> + Rcvr<M>, 
    L: Logger + Debug + Copy + Clone + Default
{
    snd_queue: Arc<BlockingQueue<M>>,
    rcv_queue: Arc<BlockingQueue<M>>,
     _p: P,
     connected: bool,
    //  shutdown : bool,
     log: L,
}
impl<P,M,L> Connector<P,M,L> where
    M: Msg + Debug + Clone + Send + Default + 'static,
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
    // pub fn shut_down(&self) {
    //     self.shutdown = true;
    // }
    pub fn new(addr: &'static str) -> std::io::Result<Connector<P,M,L>>
    where
        M: Msg + Debug + Clone + Send + Default + 'static,
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
        // let _ = stdout().flush();
        let stream = rslt.unwrap();
        let mut buf_writer = BufWriter::new(stream.try_clone()?);
        let mut buf_reader = BufReader::new(stream);
        
        let send_queue = Arc::new(BlockingQueue::<M>::new());
        let recv_queue = Arc::new(BlockingQueue::<M>::new());
        
        /*-- send thread reads input queue and sends msg --*/
        let sqm = Arc::clone(&send_queue);
        let _ = std::thread::spawn(move || {
            loop {
                let ssq = Arc::clone(&sqm);
                //print!("\n  -- dequing send msg --");
                let msg = ssq.de_q();
                //print!("\n  sending msg");
                let msg_type = msg.get_type();
                let rslt = P::buf_send_message(msg, &mut buf_writer);
                if rslt.is_err() {
                    break;
                }
                if msg_type == MessageType::END {
                    L::write("\n--terminating connector send thread--");
                    break;
                }
            }            
        });
        /*-- recv thread recvs msg (may block) and enQs for user --*/
        let rqm = Arc::clone(&recv_queue);
        let _ = std::thread::spawn(move || {
            loop {
                let srq = Arc::clone(&rqm);
                let rslt = P::buf_recv_message(&mut buf_reader, &srq);
                if rslt.is_err() {
                    L::write("\n--terminating connector receive thread--");
                    break;
                }
            }
        });
        /*-- return new Connector as std::io::Result --*/
        let me =
        Self {
            _p: P::default(),
            snd_queue: send_queue,
            rcv_queue: recv_queue,
            connected: _is_connected,
            // shutdown: false,
            log: L::default(),
        };
        Ok(me)
    }
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
    log: L
}
impl<P,L> Listener<P,L> 
where 
    P: Debug + Copy + Clone + Send + Sync + Default + Sndr<M> + Rcvr<M> + Process<M> + 'static,
    L: Logger + Debug + Copy + Clone + Default
    {    
    pub fn new() -> Listener<P,L> {
        Listener {
              p: P::default(),
              run: Arc::new(AtomicBool::new(true)),
              log: L::default(),
        }
    }
    /*-- starts thread wrapping incoming loop which often blocks --*/
    pub fn start(&mut self, addr: &'static str) -> Result<JoinHandle<()>> 
    {
        L::write(&format!("\n--starting listener on {:?}--", addr));
        let rslt = TcpListener::bind(addr);
        if rslt.is_err() {
            print!("\n  binding to {:?} failed", addr);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "listener bind failed"));
        }
        let tcpl = rslt.unwrap();
        /*-- this outer thread prevents appl from blocking waiting for connections --*/
        let handle = std::thread::spawn(move || {
            let rcv_queue = Arc::new(BlockingQueue::<M>::new());
            let run = Arc::new(AtomicBool::new(true));
            /*-- loop on incoming iterator which calls accept and so blocks --*/
            for stream in tcpl.incoming() {
                let sq = Arc::clone(&rcv_queue);
                let srun = Arc::clone(&run);

                /*-- if !run break out of accept loop --*/
                if !run.load(Ordering::Relaxed) {
                    break;
                }
                /*-- thread handles client until receiving an END or QUIT message --*/
                let strm = stream.unwrap();
                let mut buf_writer = BufWriter::new(strm.try_clone().unwrap());
                let mut buf_reader = BufReader::new(strm.try_clone().unwrap());
                let _ = std::thread::spawn(move || {
                    loop {
                        let rslt = P::buf_recv_message(&mut buf_reader, &sq);
                        if rslt.is_err() {
                            print!("\n  socket session closed abruptly");
                            break;
                        }
                        let msg = sq.de_q();
                        if msg.get_type() == MessageType::END {
                            L::write("\n--listener received END message--");
                            L::write("\n--terminating client handler loop--");           
                            break;
                        }
                        else if msg.get_type() == MessageType::QUIT {
                            srun.store(false, Ordering::Relaxed);
                            L::write("\n--listener received QUIT message--");
                            L::write("\n--terminating listener accept loop--");
                            /*---------------------------------------------
                               connect so accept returns, making false value of 
                               run visible
                            */       
                            let _rslt = TcpStream::connect(addr);
                            break;
                        }
                        /*-- used to test error handling --*/
                        else if msg.get_type() == MessageType::SHUTDOWN {
                            let _ = strm.shutdown(Shutdown::Both);
                            print!("\n  shutting down socket session");
                            break;
                        }
                        let msg = P::process_message(msg);
                        let _ = P::buf_send_message(msg, &mut buf_writer);
                    } 
                });
            }  
            L::write("\n--terminating listener thread--");  
        });
        Ok(handle)
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
