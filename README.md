# RustComm

This repository contains a Prototype for message-passing communication system  

### Figure 1. - RustComm Structure
<img src="https://JimFawcett.github.io/Pictures/RustCommConcept.jpg" width="600" />

RustComm Provides types:
- Connector&lt;P,M,L&gt;
- Listener&lt;P,L&gt;
- Message
- CommProcessing&lt;L&gt;

All application specific processing is in CommProcessing&lt;L&gt;.

See https://JimFawcett.github.io/RustComm.html for details.

### Concept:  
RustComm is a facility for sending messages between a Sender and Receiver. It uses the std::net::TcpStream and std::net::TcpListener types.  

This is a prototype for message-passing communication system. It provides three user defined types: Connector, Listener, and Message, with generic parameters M, P, and L, as shown in Fig. 1.  
  - M implements the Msg trait and represents a message to be sent between endpoints. 
  - P implements the Process<M> trait that defines message processing.
  - L implements the Logger trait that supports logging events to the console that can be turned on or off by the types supplied for L, e.g., VerboseLog and MuteLog.
  
The RustComm library:
  - Uses queued full-duplex buffered message sending and receiving
  - Each message has a fixed size header and Vec<u8> body.
  - For each Connector<P, M, L> connection, Listener<P, L> processes messages until receiving a message with MessageType::END. Listener<P, L>
    spawns a thread for each client connection and processes messages in P::process_message.
  
In this version, P::process_message echos back message with "reply" appended as reply to sender. You observe that behavior in Fig. 2.

### Goal:
The long-term goal for RustComm is to serve as a prototyping platform for various messaging and processing strategies. This version defines traits: Sndr<M>, Rcvr<M>, Process<M>, Msg, and Logger.  
  
User-defined types, M and P, are things that change as we change the message structure, defined by M and connector and listener processing defined by P. These types are defined in the rust_comm_processing crate.  

The somewhat complex handling of TcpStreams and TcpListener are expected to remain fixed. They are defined in the crate rust_comm.  
Finally, logger L provides a write method that will, using VerboseLog for L, write its argument to the console. MuteLog simply discards its argument.  

The last step in this phase of development is to add a threadpool, as shown in Fig. 1. The threadpool exists and has been combined with this code to
provide **RustCommWithThreadPool** repository.  

### Current Design:  

There are three user-defined types: Message, Connector, and Listener. Connector and Listener each use an existing component BlockingQueue<Message>.

**Message Methods:**
```
  - new() -> Message
      Create new Message with empty body and MessageType::TEXT.  
      
  - set_type(&mut self, mt: u8)
      Set MessageType member to one of: TEXT, BYTES, END.   
      
  - get_type(&self) -> MessageType
      Return MessageType member value.  
      
  - set_body_bytes(&mut self, b: Vec<u8>)
      Set body_buffer member to bytes fromb: Vec<u8>.  
      
  - set_body_str(&mut self, s: &str;)
      Set body_buffer member to bytes froms: &str.  
      
  - get_body_size(&self) -> usize
      Return size in bytes of body member.  
      
  - get_body(&self) -> &Vec<u8>
      Return body_buffer member.  
      
  - get_body_str(&self) -> String
      Return body contents as lossy String.  
      
  - clear(&self)
      clear body contents.
```
Both Connector<P, M, L> and Listener<P, L> are parameterized with L, a type satisfying a Logger trait. The package defines two types that implement the trait, VerboseLog and MuteLog that allow users to easily turn on and off event display outputs. Fig 2. uses MuteLog in both Connector<P, M, L> and Listener<P, L>.

**Connector<P, M, L> methods:**
```rust
  - new(addr: &'static str) -> std::io::Result<Connector<P,M,L>>
      Create new Connector<P,M,L> with running send and receive threads.  
      
  - is_connected(&self) -> bool
      is connected to addr?.  
      
  - post_message(&self, msg: M)
      Enqueues msg to send to connected Receiver. 
      
  - get_message(&mut self) -> M
      Reads reply message if available, else blocks.  
      
  - has_message(&self) -> bool
      Returns true if reply message is available. 
```     
**Listener<P, L> methods:**
```rust
  - new() -> Listener<P, L>
      Create new Listener<P, L>.  
      
  - start(&mut self, addr: &'static str) -> std::io::Result<JoinHandle<()>>
      Bind Listener<P,L> to addr and start listening on dedicated thread.  
```
### Operation:
This is intended to be a simple test-bed for ideas - easy to use and with very little setup and configuration.

### Build:
Download and, in a command prompt, cargo build or cargo run.  

### Status:
Expect to add file transfer capability.
