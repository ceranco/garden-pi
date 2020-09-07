/// This example shows how to use unix sockets in conjuction with serde (and json) to
/// communicate easily.
///
/// In this example, a listener waits for newline-delimited messages in a spawned thread.
/// If the message is `Msg::Msg`, it simply echoes the contents.
/// If the message is `Msg::Close` it closes the connection and deleted the socket.
///
/// The client sends 10 messages in a loop, then sends the `Msg::Close` msg.
use serde::{Deserialize, Serialize};
use std::{
    io::{BufRead, BufReader, Write},
    os::unix::net::{UnixListener, UnixStream},
    thread,
};

const SOCKET: &str = "/tmp/unix-sockets-example.sock";

#[derive(Serialize, Deserialize)]
enum Msg {
    Msg(String),
    Close,
}

fn main() {
    let listener = UnixListener::bind(SOCKET).unwrap();
    let mut client = UnixStream::connect(SOCKET).unwrap();

    let handle = thread::spawn(move || {
        println!("Server: Listening!");

        'listen: for stream in listener.incoming() {
            let stream = BufReader::new(stream.unwrap());
            for line in stream.lines() {
                let msg: Msg = serde_json::from_str(&line.unwrap()).unwrap();
                match msg {
                    Msg::Msg(msg) => println!("{:?}", msg),
                    Msg::Close => break 'listen,
                }
            }
        }

        println!("Server: Closing listener!");
        std::fs::remove_file(SOCKET).unwrap();
    });

    println!("Client: Sending messages.");
    for i in 0..10 {
        let msg = Msg::Msg(format!("Message no. {}", i));
        let mut msg = serde_json::to_string(&msg).unwrap();
        msg.push('\n');

        client.write_all(msg.as_bytes()).unwrap();
    }

    println!("Client: Sending close message.");
    let mut msg = serde_json::to_string(&Msg::Close).unwrap();
    msg.push('\n');
    client.write_all(msg.as_bytes()).unwrap();

    handle.join().unwrap();
}
