/////////////////////////////////////////////////////////////
// rust_thread_pool::lib.rs - threadpool wit BlockingQueue // 
//                                                         //
// Jim Fawcett, https://JimFawcett.github.com, 29 Jun 2020 //
/////////////////////////////////////////////////////////////
/*
   There are two undefined methods for ThreadPool<M>
   that need to be implemented before this design is
   complete, e.g.:
   - post_work_item posts a function object to input queue
   - get_message retrieves results from an output queue
*/
#![allow(dead_code)]
use std::fmt::*;
use rust_blocking_queue::*;
use std::thread::*;
use std::sync::*;
use std::default::{Default};

#[derive(Debug)]
pub struct ThreadPool<M> 
{
    sibq: Arc<BlockingQueue<M>>,
    sobq: Arc<BlockingQueue<M>>,
    thrd: Vec<Option<JoinHandle<()>>>
    /* see note below about Option */
}
impl<M> ThreadPool<M> 
where M: Send + 'static
{
    /*-----------------------------------------------------
      construct threadpool, starting nt threads,
      provide threadpool processing as f:F in new 
    */
    pub fn new<F>(nt:u8, f:F) -> ThreadPool<M> 
    where F: FnOnce(&BlockingQueue<M>, &BlockingQueue<M>) -> () + Send + 'static + Copy
    {
        /* safely share BlockingQueue with Arc */
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
            let soq = Arc::clone(&soqm);
            let handle = std::thread::spawn( move || { 
                f(&siq, &soq);  // thread_pool_processing
            });
            vt.push(Some(handle));
        }
        Self { // return newly created threadpool
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
    /*-- return results to caller --*/
    pub fn get(&mut self) -> M 
    where M:Debug + Default {
        let m:M = self.sobq.de_q();
        m
    }
    /*-- return results to caller --*/
    pub fn done(&mut self) -> bool {
        false
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
