
// https://doc.rust-lang.org/book/ch20-01-single-threaded.html
use std::io;
use std::io::{prelude::*};
use std::net::{TcpListener, TcpStream};
use std::collections::HashMap;
use std::cell::RefCell;

//const FTP_PORT:u16 = 2121;
const HELLO : &[u8] = b"220 Hello from a silly ftp server..\r\n";
const USERNAME_OK : &[u8] = b"331 User name okay, need password.\r\n";
const PASSWORD_OK : &[u8] = b"230 User logged in, proceed.\r\n";
const BYE : &[u8] = b"200 bye!\r\n";
const SYST_RESPONSE : &[u8] = b"215 UNIX Type: L8\r\n";

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

struct State
{
    pub ctrl : RefCell<TcpStream>,
    pub data : Option<RefCell<TcpListener>>
}

impl State {
    pub fn new(ctrl: TcpStream) -> Self {
        State {
            ctrl: RefCell::new(ctrl),
            data: None
        }
    }
}
fn handle_syst(_args: &str, state: &mut State)
{
    println!("handle_syst");
    let _ = state.ctrl.borrow_mut().write(SYST_RESPONSE);
}

fn handle_pasv( _args: &str, state: &mut State)
{
    println!("handle_pasv");

    let bindto = state.ctrl.borrow_mut().local_addr().unwrap().ip().to_string() + ":0";

    let listener = TcpListener::bind(bindto).unwrap();

//    let socket = listener.local_addr().unwrap();

    state.data = Some(RefCell::new(listener));

    // local_addr to find the IP address of the socket
    let socket = state.data.as_ref().unwrap().borrow().local_addr().unwrap();
    let response = format!("227 Entering Passive Mode ({},{},{})\r\n",
                           socket.ip().to_string().replace(".", ","),
                           socket.port() / 256,
                           socket.port() % 256
                           );
    let _ = state.ctrl.borrow_mut().write_all(response.as_bytes());
}

fn handle_type( args: &str, state: &mut State)
{
    // created by AI assistant :-O
    let args = args.trim();
    if args == "A" {
        state.ctrl.borrow_mut().write_all(b"200 Type set to ASCII.\r\n").unwrap();
    } else if args == "I" {
        state.ctrl.borrow_mut().write_all(b"200 Type set to Binary.\r\n").unwrap();
    } else {
        state.ctrl.borrow_mut().write_all(b"504 Command not implemented for that parameter.\r\n").unwrap();
    }
}
fn handle_stor(args: &str, state: &mut State)
{
    state.ctrl.borrow_mut().write_all(b"125 Data connection already open; transfer starting.").unwrap();

    let mut file = std::fs::File::create(args).unwrap();

    let accepting = state.data.as_ref().unwrap().borrow_mut().accept();

    let mut data_stream = accepting.unwrap();

    println!("connection from {}", data_stream.1.to_string());
    let result = std::io::copy(&mut data_stream.0, &mut file).unwrap();
    println!("copied {} bytes", result);

    let _ = state.ctrl.borrow_mut().write_all(b"226 Closing data connection.");
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

    let mut map: HashMap<String, fn(&str, &mut State)> = HashMap::new();
    map.insert("SYST".to_string(), handle_syst);
    map.insert("PASV".to_string(), handle_pasv);
    map.insert("TYPE".to_string(), handle_type);
    map.insert("STOR".to_string(), handle_stor);

    let mut state = State::new(stream);

    loop {
        let s = readline(&mut state.ctrl.borrow_mut()).unwrap();
        let (cmd, arg)  = parse_command(&s).unwrap();

        println!("command={:?}", cmd);
        if cmd == "QUIT" {
            let _ = state.ctrl.borrow_mut().write(BYE);
            break;
        }

        if let Some(func) = map.get(&cmd) {
            func(&arg, &mut state);
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