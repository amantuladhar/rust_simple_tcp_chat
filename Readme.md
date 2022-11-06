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

## Multiple Client



