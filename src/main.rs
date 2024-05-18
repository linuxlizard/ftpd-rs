use std::collections::HashMap;
// https://doc.rust-lang.org/book/ch20-01-single-threaded.html
use std::io;
use std::io::{prelude::*};
use std::net::{TcpListener, TcpStream};

//const FTP_PORT:u16 = 2121;
const HELLO : &[u8] = b"220 Hello from a silly ftp server..\r\n";
const USERNAME_OK : &[u8] = b"331 User name okay, need password.\r\n";
const PASSWORD_OK : &[u8] = b"230 User logged in, proceed.\r\n";
const BYE : &[u8] = b"200 bye!\r\n";

const CR:u8 = b'\r';
const LF:u8 = b'\n';

fn readline(stream: &mut TcpStream) -> io::Result<String>
{
    let mut s = String::new();

    loop {
        let mut b: [u8; 1] = [0];

        stream.read_exact(&mut b)?;
        println!("read b={:?}", b);
        if b[0] == CR {
            continue;
        }
        if b[0] == LF {
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

    Ok(s.trim().to_string())
}

fn parse_command(s : &str) -> Option<(String, String)>
{
    let p = s.find(' ');

    let pos = match p {
        Some(pos) => pos,
        None => return Some((s.to_string(), "".to_string()))
    };

    let (left,right) = s.split_at(pos);

    let command = left;
    let argument = right.trim().to_string();

    println!("command={} argument={}", command, argument);
    Some((command.to_string(), argument))
}

fn handle_syst(args: &str, stream: &mut TcpStream)
{
    println!("handle_syst");
    let _ = stream.write(b"215 UNIX Type: L8\r\n");
}

fn handle_pasv( args: &str, stream: &mut TcpStream)
{
    println!("handle_pasv");
}

fn handle_connection(mut stream: TcpStream) 
{
    let _ = stream.write(HELLO);

    let s = readline(&mut stream).unwrap();
    let (cmd, arg) = parse_command(&s).unwrap();
    if cmd != "USER" {
        return;
    }
    println!("username={:?}", arg);
    let _ = stream.write(USERNAME_OK);

//    let _ = stream.write(b"password: ");
    let s = readline(&mut stream).unwrap();
    let (cmd, arg)  = parse_command(&s).unwrap();
    if cmd != "PASS" { return; }
    let _ = stream.write(PASSWORD_OK);
    println!("password={:?}", arg);

    let mut map: HashMap<String, fn(&str, &mut TcpStream)> = HashMap::new();
    map.insert("SYST".to_string(), handle_syst);
    map.insert("PASV".to_string(), handle_pasv);

    loop {
        let s = readline(&mut stream).unwrap();
        let (cmd, arg)  = parse_command(&s).unwrap();

        println!("command={:?}", cmd);
        if cmd == "QUIT" {
            let _ = stream.write(BYE);
            break;
        }

        if let Some(func) = map.get(&cmd) {
            func(&arg, &mut stream);
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    // Test block for parse_command
    #[test]
    fn test_parse_command()
    {
        println!("--- Test for parse_command ---");

        // Test cases
        let test_commands = vec![
            "LIST -l",
            "USER anonymous",
            "PASS password",
            "QUIT",
            "FEAT",
        ];

        for command in test_commands {
            println!("Testing command: {}", command);
            match parse_command(&command) {
                Some((cmd, arg)) => {
                    println!("Result: cmd={} arg={}", cmd, arg);
                },
                None => {
                    println!("Failed to parse the command {}", command);
                }
            }
        }

        println!("--- End of test for parse_command ---");
    }

    // ...
}