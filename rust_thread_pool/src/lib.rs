/////////////////////////////////////////////////////////////
// rust_thread_pool::lib.rs - threadpool wit BlockingQueue // 
//                                                         //
// Jim Fawcett, https://JimFawcett.github.com, 29 Jun 2020 //
/////////////////////////////////////////////////////////////
/*
   ThreadPool<M> instances start a specified number of
   threads, each of which executes a processing function.
*/
#![allow(dead_code)]
use std::fmt::*;
use rust_blocking_queue::*;
use std::thread::*;
use std::sync::*;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug)]
pub struct ThreadPool<M> 
{
    run: Arc<AtomicBool>,
    sibq: Arc<BlockingQueue<M>>,
    sobq: Arc<BlockingQueue<M>>,
    thrd: Vec<Option<JoinHandle<()>>>,
    /* see note below about Option */
}
impl<M> ThreadPool<M> 
where M: Send + 'static
{
    /*-----------------------------------------------------
      construct threadpool:
      - start nt threads,
      - pass processing function that accepts
        ThreadPool blocking queue and stopping
        signal
    */
    pub fn new<F>(nt:u8, f:F) -> ThreadPool<M> 
    where F: FnOnce(&BlockingQueue<M>, &Arc<AtomicBool>) -> () + Send + 'static + Copy
    {
        let run_ref = Arc::new(AtomicBool::new(true));
        let siqm = Arc::new(BlockingQueue::<M>::new());
        let soqm = Arc::new(BlockingQueue::<M>::new());
        let mut vt = Vec::<Option<JoinHandle<()>>>::new();
        /* start nt threads */
        for _i in 0..nt {
            /*----------------------------------------------- 
              ref sq to master shared queue (siqm) is captured
              by thread proc closure 
            */
            let siq = Arc::clone(&siqm);
            let run = Arc::clone(&run_ref);
            let handle = std::thread::spawn( move || { 
                f(&siq, &run);  // thread_pool_processing
            });
            vt.push(Some(handle));
        }
        Self { // return newly created threadpool
            run: run_ref,
            sibq: siqm,
            sobq: soqm,
            thrd: vt, 
        }
    }
    /*-- wait for threads to finish --*/
    pub fn wait(&mut self) {
        for handle in &mut self.thrd {
            let _ = handle.take().unwrap().join();
            /*
              This is a hack!
              Without the Option, wrapping threadhandle, can't move threadhandle
              out of Vec<JoinHandle<()>>, so error in line above. 
              
              Can move out of the option as long as we replace
              the moved value (take swaps None for Some in option).

              I was stumpted until I saw this link.  Apparently a well known hack.
              https://users.rust-lang.org/t/spawn-threads-and-join-in-destructor/1613
            */
        }
    }
    /*-- post to ThreadPool queue --*/
    pub fn post(&mut self, _msg:M) 
    where M:Debug {
        self.sibq.en_q(_msg);
    }
    /*-----------------------------------------------------
      signals threads to terminate
      - run() must be tested in thread processing function
        used for threadpool threads.
      - See test1.rs for an example.
    */
    pub fn stop(&mut self) {
        self.run.store(false, Ordering::Relaxed);
    }
    /*-- test stopping signal --*/
    pub fn run(&self) -> &Arc<AtomicBool> {
        &self.run
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new() {
        let test = |bq:&BlockingQueue<String>| { 
            let msg = bq.de_q();
            print!("\n  {:?}", msg);
        };
        let mut tp = ThreadPool::<String>::new(2, test);
        let msg = "test message".to_string();
        tp.post(msg);
        tp.post("quit".to_string());
        tp.wait();
    }
}
