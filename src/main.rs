
// https://doc.rust-lang.org/book/ch20-01-single-threaded.html
use std::io;
use std::io::{prelude::*};
use std::net::{TcpListener, TcpStream};
use std::collections::HashMap;
use std::cell::RefCell;

//const FTP_PORT:u16 = 2121;
const MSG_HELLO: &[u8] = b"220 Hello from a silly ftp server..\r\n";
const MSG_USERNAME_OK: &[u8] = b"331 User name okay, need password.\r\n";
const MSG_PASSWORD_OK: &[u8] = b"230 User logged in, proceed.\r\n";
const MSG_BYE: &[u8] = b"200 bye!\r\n";
const MSG_SYST_RESPONSE: &[u8] = b"215 UNIX Type: L8\r\n";
const MSG_CLOSING : &[u8] = b"226 Closing data connection.\r\n";
const MSG_DATA_TRANSFER_STARTING :&[u8] = b"125 Data connection already open; transfer starting.\r\n";

const CR:u8 = b'\r';
const LF:u8 = b'\n';

fn readline(stream: &mut TcpStream) -> io::Result<String>
{
    let mut s = String::new();

    loop {
        let mut b: [u8; 1] = [0];

        stream.read_exact(&mut b)?;
        println!("read b={:?} c={:?}", b, b[0] as char);
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

    pub fn open_data_port(&mut self ) -> io::Result<usize>
    {
        match self.data {
            None => {
                let bindto = self.ctrl.borrow_mut().local_addr().unwrap().ip().to_string() + ":0";
                let listener = TcpListener::bind(bindto).unwrap();
                self.data = Some(RefCell::new(listener));
                // TODO handle ASCII mode
//                write_msg(b"150 Open BINARY mode data connection.\r\n", self)
                Ok(0)
            },
            Some(_) => {
                write_msg(MSG_DATA_TRANSFER_STARTING, self)
            }
        }
    }

    pub fn close_data_port(&mut self)
    {
        self.data = None;
    }
}

fn write_msg(msg: &[u8], state: &mut State) -> io::Result<usize>
{
    state.ctrl.borrow_mut().write(msg)
}

fn handle_syst(_args: &str, state: &mut State) -> io::Result<usize>
{
    println!("handle_syst");
    write_msg(MSG_SYST_RESPONSE, state)
}

fn handle_pasv( _args: &str, state: &mut State) -> io::Result<usize>
{
    println!("handle_pasv");
/*
    let bindto = state.ctrl.borrow_mut().local_addr().unwrap().ip().to_string() + ":0";

    let listener = TcpListener::bind(bindto).unwrap();

    state.data = Some(RefCell::new(listener));
*/
    state.close_data_port();
    state.open_data_port()?;

    // local_addr to find the IP address of the socket
    let socket = state.data.as_ref().unwrap().borrow().local_addr().unwrap();
    let response = format!("227 Entering Passive Mode ({},{},{})\r\n",
                           socket.ip().to_string().replace(".", ","),
                           socket.port() / 256,
                           socket.port() % 256
                           );
    write_msg(response.as_bytes(), state)
}

fn handle_type( args: &str, state: &mut State) -> io::Result<usize>
{
    // created by AI assistant :-O
    let args = args.trim();
    if args == "A" {
        write_msg(b"200 Type set to ASCII.\r\n", state)
    } else if args == "I" {
        write_msg(b"200 Type set to Binary.\r\n", state)
    } else {
        write_msg(b"504 Command not implemented for that parameter.\r\n", state)
    }
}
fn handle_stor(args: &str, state: &mut State) -> io::Result<usize>
{
    let mut file = std::fs::File::create(args).unwrap();

//    let _ = state.open_data_port();
    write_msg(b"150 Open BINARY mode data connection.\r\n", state)?;

    let accepting = state.data.as_ref().unwrap().borrow_mut().accept();

    let mut data_stream = accepting.unwrap();

    println!("connection from {}", data_stream.1.to_string());
    let result = std::io::copy(&mut data_stream.0, &mut file).unwrap();
    println!("copied {} bytes", result);

    // close the data socket
    state.close_data_port();

    write_msg(MSG_CLOSING, state)
}
fn handle_list(_args: &str, state: &mut State) -> io::Result<usize>
{
    println!("handle_list");

    state.open_data_port()?;

    let accepting = state.data.as_ref().unwrap().borrow_mut().accept();
    let mut data_stream = accepting.unwrap();
    println!("connection from {}", data_stream.1.to_string());

    let _ = data_stream.0.write_all(b"stuff stuff stuff\r\n\r\n");

    state.close_data_port();

    println!("handle_list done");

    write_msg(MSG_CLOSING, state)
}

fn handle_connection(mut stream: TcpStream) 
{
    let _ = stream.write(MSG_HELLO);

    let s = readline(&mut stream).unwrap();
    let (cmd, arg) = parse_command(&s).unwrap();
    if cmd != "USER" {
        return;
    }
    println!("username={:?}", arg);
    let _ = stream.write(MSG_USERNAME_OK);

//    let _ = stream.write(b"password: ");
    let s = readline(&mut stream).unwrap();
    let (cmd, arg)  = parse_command(&s).unwrap();
    if cmd != "PASS" { return; }
    let _ = stream.write(MSG_PASSWORD_OK);
    println!("password={:?}", arg);

    let mut map: HashMap<String, fn(&str, &mut State) -> io::Result<usize>> = HashMap::new();
    map.insert("SYST".to_string(), handle_syst);
    map.insert("PASV".to_string(), handle_pasv);
    map.insert("TYPE".to_string(), handle_type);
    map.insert("STOR".to_string(), handle_stor);
    map.insert("LIST".to_string(), handle_list);

    let mut state = State::new(stream);

    loop {
        let s = readline(&mut state.ctrl.borrow_mut()).unwrap();
        let (cmd, arg)  = parse_command(&s).unwrap();

        println!("command={:?}", cmd);
        if cmd == "QUIT" {
            let _ = state.ctrl.borrow_mut().write(MSG_BYE);
            break;
        }

        if let Some(func) = map.get(&cmd) {
            let _ = func(&arg, &mut state);
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