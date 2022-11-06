use std::net::SocketAddr;

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
    sync::broadcast,
};

#[tokio::main]
async fn main() {
    let tcp_listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    let (channel_send, _) = broadcast::channel::<(String, SocketAddr)>(10);
    loop {
        let (mut socket, addr) = tcp_listener.accept().await.unwrap();
        let channel_send = channel_send.clone();
        let mut channel_read = channel_send.subscribe();
        tokio::spawn(async move {
            let (socket_reader, mut socket_writer) = socket.split();

            let mut br = BufReader::new(socket_reader);
            let mut message = String::new();

            loop {
                tokio::select! {
                    num_of_bytes = br.read_line(&mut message) => {
                        channel_send.send((message.clone(), addr)).unwrap();
                        message.clear();
                    }
                    recv_msg = channel_read.recv() => {
                        let (recv_msg, o_addr) = recv_msg.unwrap();
                        if addr != o_addr {
                            socket_writer.write_all(recv_msg.as_bytes()).await.unwrap();
                        }
                    }
                }
            }
        });
    }
}
