// https://doc.rust-lang.org/book/ch20-01-single-threaded.html
use std::io;
use std::io::{prelude::*};
use std::net::{TcpListener, TcpStream};

//const FTP_PORT:u16 = 2121;

fn readline(stream: &mut TcpStream) -> io::Result<String>
{
    let mut buf: [u8; 1] = [0;1];

    loop {

        let read_result = stream.read(&mut buf);
        match read_result {
            Ok(size) => {
                println!("read size={} char={:#x}", size, buf[0]);
                if size == 0 { break; }
            },
            Err(_err) => { panic!("failed to read"); }
        }
    }

    Ok("todo".to_string())
}

fn handle_connection(mut stream: TcpStream) 
{
    let _ = stream.write(b"username: ");

    let username = readline(&mut stream).unwrap();
    println!("username={:?}", username);

    let _ = stream.write(b"password: ");
    let password = readline(&mut stream).unwrap();
    println!("password={:?}", password);

    println!("connection done");
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:2121").unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        println!("connection={:?}", stream);
        handle_connection(stream);
    }
}

