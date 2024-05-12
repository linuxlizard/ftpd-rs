// https://doc.rust-lang.org/book/ch20-01-single-threaded.html
use std::io;
use std::io::{prelude::*};
use std::net::{TcpListener, TcpStream};

//const FTP_PORT:u16 = 2121;
const HELLO : &[u8] = b"220 Hello from a silly ftp server..\r\n";
const USERNAME_OK : &[u8] = b"331 User name okay, need password.\r\n";
const PASSWORD_OK : &[u8] = b"230 User logged in, proceed.\r\n";

fn readline(stream: &mut TcpStream) -> io::Result<String>
{
    let mut s = String::new();

    loop {
        let mut b: [u8; 1] = [0];

        stream.read_exact(&mut b)?;
        println!("read b={:?}", b);
        if b[0] == 0x0a {
            break;
        }
        s.push(b[0] as char);
//        match read_result {
//            Ok(()) => {
//                println!("read b={:?}", b);
//            },
//            Err(err) => { panic!("failed to read"); }
//        }
    }

    Ok(s)
}

fn handle_connection(mut stream: TcpStream) 
{
    let _ = stream.write(HELLO);

    let username = readline(&mut stream).unwrap();
    println!("username={:?}", username);
    let _ = stream.write(USERNAME_OK);

//    let _ = stream.write(b"password: ");
    let password = readline(&mut stream).unwrap();
    let _ = stream.write(PASSWORD_OK);
    println!("password={:?}", password);

    println!("connection done");
    loop {
        let cmd = readline(&mut stream).unwrap();
        println!("command={:?}", cmd);
    }

}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:2121").unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        println!("connection={:?}", stream);
        handle_connection(stream);
    }
}

