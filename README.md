# RustComm

https://JimFawcett.github.io/RustComm.html

rust_comm::lib.rs Facilities:
-----------------------------
Prototype for message-passing communication system
Provides two user defined types: Sender and Receiver. 
 - Uses unbuffered, unqueued full-duplex message sending and 
   receiving
 - Each message has a fixed size header and Vec<u8> body.
 - For each Sender connection, Receiver processes messages
   until receiving a message with MessageType::END.
 - Receiver spawns a thread for each client connection and
   processes messages in an external handle_client function.
 - In this version, handle_client only displays message body
   to console.  It does not send back a replay message.

Expected Changes and Additions:
-------------------------------
 - Add reply messages to this demo.
 - Add Sender queue and a threadpool in Receiver
 - Convert to buffered reads and writes
 - Add user-defined Comm type that composes a Sender and a
   Receiver.  
 - Support interhangeably Messages that use references to
   external facilities for defining message body contents
   rather than packing into message. 

