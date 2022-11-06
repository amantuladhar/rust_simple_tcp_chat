use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
};

#[tokio::main]
async fn main() {
    let tcp_listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    let (mut socket, addr) = tcp_listener.accept().await.unwrap();
    let (socket_reader, mut socket_writer) = socket.split();

    let mut br = BufReader::new(socket_reader);
    let mut message = String::new();

    loop {
        let num_of_bytes_read = br.read_line(&mut message).await.unwrap();
        socket_writer.write_all(message.as_bytes()).await.unwrap();
        message.clear();
    }
}
