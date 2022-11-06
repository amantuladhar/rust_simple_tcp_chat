# Simple TCP Chat Server with Rust

## Add tokio as dependencies
> tokio = { version = "1", features = ["full"] }
- For our purposes we are downloading full tokio binary but we can pick and choose


## Make main method `async`
```rust
async fn main() {
    // body
}
```
- Rust will complain as main fn cannot be async

## `tokio` to rescue
```rust
#[tokio::main]
async fn main() {
    // body
}
```
- `#[tokio::main]` macro will wrap your main function an run it inside tokio runtime

## Create a TCPListener
```rust
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let tcp_listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
}
```
- We will use `TcpListener` to listen to our desired port. In our case we are listening to port `8080`
- `TcpListener` will return a `Future` which we will need to await

## Wait for client to connect
```rust
// ...
async fn main () {
    //...
    let (socket, addr) = tcp_listener.accept().await.unwrap();
}
// ...
```
- We use TCP listener we created before to wait for client to connect.
- `accept()` function returns the `Future`. We are using await here to block the call until client connects

## (Run 1)
> cargo run
- Our app will wait for client to connect to port 8080
- To test our server we can use `telnet` command

> telnet localhost 8080
```
Trying ::1...
telnet: connect to address ::1: Connection refused
Trying 127.0.0.1...
Connected to localhost.
Escape character is '^]'.
Connection closed by foreign host.
```
- Our connection was successful as you can see
- But as soon as our connection was established our blocking code started to execute, and because we didn't have any code our program exited.

## Accept the message from client and print
```rust 
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::TcpListener,
};
//...
let mut br = BufReader::new(socket); // 1
let mut message = String::new(); // 2
let num_of_bytes_read = br.read_line(&mut message).await.unwrap(); // 3
println!("{message} :: {num_of_bytes_read}"); // 4
```
- `1` Create a BufReader from tokio to read the message from socket
- `2` Create a container to store the message in
- `3` use BufReader to read the message.
- `4` print the message to server

## (Attempt 1) Print the message on client instead of server
```rust
// ERROR - Does not compile
let tcp_listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
let (mut socket, addr) = tcp_listener.accept().await.unwrap(); // 1

let mut br = BufReader::new(socket); // 2
let mut message = String::new();
let num_of_bytes_read = br.read_line(&mut message).await.unwrap();

socket.write_all(message.as_bytes()).await.unwrap() // 3
```
- If we want to send the message back to client we need to use socket `write_all` method
- `1` We need to define socket as `mut` as we need to use write now
- `2` When creating BufReader we moved the socket ownership to BufReader
- `3` Rust doesn't allow us to use the reference that is already moved

## (Attempt 2) - Split socket to reader and writer for fine grain control
```rust
    let tcp_listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    let (mut socket, addr) = tcp_listener.accept().await.unwrap();
    let (socket_reader, mut socket_writer) = socket.split(); // 1

    let mut br = BufReader::new(socket_reader); // 2
    let mut message = String::new();
    let num_of_bytes_read = br.read_line(&mut message).await.unwrap();

    socket_writer.write_all(message.as_bytes()).await.unwrap() // 3
```
- `1` split the socket into reader and writer
- `2` Pass only socket_reader into BufReader. Now BufReader takes ownership of reader only
- `3` Use socket_writer to write to client. Because writer ownership is untouched we can finally make rust happy.

## (Run 2)
- Run the app with `cargo run`
- Then do `telnet localhost 8080`
```
Trying ::1...
telnet: connect to address ::1: Connection refused
Trying 127.0.0.1...
Connected to localhost.
Escape character is '^]'.
123 👈 Client message send to server
123 👈 Server printing it back
Connection closed by foreign host.
```
- Note that our app still terminates after only one message
- Hopefully you know this is expected as we we have not implemented any loops yet

## Accept multiple messages
```rust
    loop {
        let num_of_bytes_read = br.read_line(&mut message).await.unwrap();
        socket_writer.write_all(message.as_bytes()).await.unwrap()
    }
```
- If we wrap the part where we wait for message and send it back, we will be able to send multiple message now

## (Run 3)
```
Trying ::1...
telnet: connect to address ::1: Connection refused
Trying 127.0.0.1...
Connected to localhost.
Escape character is '^]'.
1 👈 Send
1 👈 Receive
2 👈 Send
1 👈 Receive
2 👈 Receive
3 👈 Send
1 👈 Receive
2 👈 Receive
3 👈 Receive
Connection closed by foreign host.
```
- You can see now we can send multiple messages
- But we have a weired behavior. Everytime we receive the message, server sends us all of previous messages as well.
- This happens because by defaut BufReader doesn't remove the previous content it read. It is up to us to delete reset the content.
- Why? BufReader can be use to read the large file line by line. We don't want to delete the previous line when we read new line.

## Reset the previous messages so client doesn't get duplicate message
```rust
    loop {
        let num_of_bytes_read = br.read_line(&mut message).await.unwrap();
        socket_writer.write_all(message.as_bytes()).await.unwrap();
        message.clear(); // 1
    }
```
- Just call clear() method on our message container

## (Attempt 1) Multiple Client
```rust
    loop { // 1
        let (mut socket, addr) = tcp_listener.accept().await.unwrap(); // 2
        //...
        loop {
            let num_of_bytes_read = br.read_line(&mut message).await.unwrap();
            socket_writer.write_all(message.as_bytes()).await.unwrap();
            message.clear();
        }
    }
```
- `1` wrap the whole code in loop - starting from where we start to accept client
- `2` this is where we accept client
- Notice this doesn't quite work well, if you follow code code
- We have not introduced the concurrent code yet. As it stands right now, our code will
    - Wait for client_1 connect
    - Wait a message from client_1
    - Send message back to client_1
    - Wait a message from client_1
    - ...
- Our code nevers breaks the inner loop
- To make this work we need to make our code asyncronous

## (Attempt 2) Multiple Client - tokio to rescue AGAIN
```rust
    loop {
        let (mut socket, addr) = tcp_listener.accept().await.unwrap();

        tokio::spawn(async move { // 1
            let (socket_reader, mut socket_writer) = socket.split();

            let mut br = BufReader::new(socket_reader);
            let mut message = String::new();

            loop {
                let num_of_bytes_read = br.read_line(&mut message).await.unwrap();
                socket_writer.write_all(message.as_bytes()).await.unwrap();
                message.clear();
            }
        });
    }
```
- `2` we use tokio spawn method to do our work in different thread.
- This way our code doesn't wait for inner loop to finish (which we know it never exits)
- Now the code flow will look something like this
    - wait for client_1
    - as soon as client_1 is connect - process client_1 message on different thread
    - don't wait for client_1 work, start to wait for different client
    - after client_2 connects, client_2 will get a new thread again

```bash
# Client 1
Trying ::1...
telnet: connect to address ::1: Connection refused
Trying 127.0.0.1...
Connected to localhost.
Escape character is '^]'.
1 👈 Send
1 👈 Receive
1 👈 Send
1 👈 Receive

# Client 2
Trying ::1...
telnet: connect to address ::1: Connection refused
Trying 127.0.0.1...
Connected to localhost.
Escape character is '^]'.
2 👈 Send
2 👈 Receive
2 👈 Send
2 👈 Receive
```
- Note client are still isolated, they are not communicating with one another
- But both client can chat with one another concurrently.

## Clients talk to each other
- To make client talk to each other needs a bit of work
- For them to communicate we need to
    - create a broadcast channel
    - send message to broadcast channel
    - receive message from broadcast channel
- for this to work, all client that are connected needs to use same broadcast channel

## Create a broadcast channel where client can send and receive message
```rust
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
    sync::broadcast,
};
// ...
async fn main() {
    let tcp_listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    let (channel_send, channel_read) = broadcast::channel::<String>(10); // 1
    loop {
    //...
```
- `1` Here we are creating a channel that can send and receive strings

## Send message to broadcast channel when message is received
```rust
    // ERROR - doesn't compile
    let (channel_send, channel_read) = broadcast::channel::<String>(10);
    loop {
        let (mut socket, addr) = tcp_listener.accept().await.unwrap();
        tokio::spawn(async move {
            //...
            loop {
                let num_of_bytes_read = br.read_line(&mut message).await.unwrap();
                channel_send.send(message.clone()); // 1
                //...
```
- `1` Send a message to broadcast. This will fail because rust will yell at you for using shared value inside of loop.
- Note we are sending clone of message to make sure we don't transfer ownership on message variable to send() method.
- Thankfully channel_send as a clone() method to borrow the copy. Note internally Sender<T> is using Arc to manage borrow state for concurrent code.

```rust
    //...
    let (channel_send, channel_read) = broadcast::channel::<String>(10);
    loop {
        let (mut socket, addr) = tcp_listener.accept().await.unwrap();
        let channel_send = channel_send.clone(); // 1
        tokio::spawn(async move {
            let (socket_reader, mut socket_writer) = socket.split();
    //...
```
- `1` clone() the channel_send to borrow the referance. 

## Read message from broadcast channel
```rust
  let (mut socket, addr) = tcp_listener.accept().await.unwrap();
  let channel_send = channel_send.clone();
  let channel_read = channel_send.subscribe(); // 1
```
- We subscribe to broadcast channel using sender.
- It is different than how we clone channel_send but it is the API

```rust
loop {
    let num_of_bytes_read = br.read_line(&mut message).await.unwrap();
    channel_send.send(message.clone()).unwrap();

    let recv_msg = channel_read.recv().await.unwrap(); // 1
    socket_writer.write_all(recv_msg.as_bytes()).await.unwrap(); // 2
    message.clear();
}
```
- `1` Read the message from channel instead of using `message` where message already exist. Why? We will get answer to this in a bit
- `2` Write to client the message read from broadcast channel

## (Run 4)
- If you run the app and connect 2 client we will see weired behavior
- If you send multiple message on all clients, you can see we are technically sending the message between clients
- But the message are being transfered to client in weired order
- If you look at the code, I hope you can see why
- We changed our code to use broadcast, but technically we have not changed anything yet
- Out code still reads the message from client and only after the read is complete client receives the message
```rust
loop {
    let num_of_bytes_read = br.read_line(&mut message).await.unwrap(); // 1
    channel_send.send(message.clone()).unwrap();

    let recv_msg = channel_read.recv().await.unwrap(); // 2
    socket_writer.write_all(recv_msg.as_bytes()).await.unwrap();
    message.clear();
}
```
- `1` Each client runs this code first. That is, it waits for client to send message
- `2` This is where we receive the message
- After message is received from client we broadcast to our shared channel
- Note all client will have same behavior. To get the message, client needs to send something
- To fix this we have to somehow make sending and receiving message asyncronous too

## Making sending and receiving message asyncronous to each other
- There are multiple ways to go about this issue

Approach 1

- We can technically spawn a new thread with `tokio::spawn` to do these task on their individual threads.
- This would technically solve our issue, but if we go that route it will take a bit of errort because of how tokio (rust async) spawns a thread.
- In short we will get into lifetime issues, as tokio thread expects all variable we `move` inside a thead has a `static` lifetime. But our channel, buf_reader and other stuff are not `static` lifetime

Approach 2

- tokio has a helper macro `tokio::select!`.
- This is almost similar to golang select statement for their channel
- In simple terms we can define multiple futures and which ever comes first gets executed
```rust
loop {
    tokio::select! {
        num_of_bytes = br.read_line(&mut message) => { // 1
            channel_send.send(message.clone()).unwrap();
            message.clear();
        }
        recv_msg = channel_read.recv() => { // 2
            let recv_msg = recv_msg.unwrap();
            socket_writer.write_all(recv_msg.as_bytes()).await.unwrap();
        }
    }
}
```
- `1` define the first future. Notice we don't have to use `await`
- `2` second future to read the message
- `tokio::select!` will execute one future - whichever resolves first
- because we are wrapping `tokio::select!` on `loop` we will perform same task over and over again
- Note that `tokio::select!` is blocking, it blocks the code until one of the futures defines resolves
- In our case, out inner loop waits until either we receive a message from client OR we receive a message from our broadcast channel
- Once message is received we execute the code. Note that loop will restart only after a block has finished executing.

## (Run 5)
- If you run the app with few clients you can now see they can pass message to one another

## Client to not receive their own message

