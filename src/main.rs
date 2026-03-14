
// https://doc.rust-lang.org/book/ch20-01-single-threaded.html
use std::io;
use std::io::{prelude::*};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::collections::HashMap;
use std::thread;

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
fn readline(reader: &mut BufReader<TcpStream>) -> std::io::Result<String> {
    let mut s = String::new();
    reader.read_line(&mut s)?;
    Ok(s.trim_end_matches(|c| c == '\r' || c == '\n').to_string())
}

fn parse_command(s: &str) -> Option<(String, String)> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    let mut parts = s.splitn(2, ' ');
    let command = parts.next()?.to_uppercase();
    let argument = parts.next().unwrap_or("").to_string();

    println!("command={} argument={}", command, argument);
    Some((command, argument))
}

struct State {
    pub ctrl: TcpStream,
    pub data_listener: Option<TcpListener>,
}

impl State {
    pub fn new(ctrl: TcpStream) -> Self {
        State {
            ctrl,
            data_listener: None,
        }
    }

    pub fn open_data_port(&mut self) -> std::io::Result<()> {
        if self.data_listener.is_some() {
            return Ok(());
        }

        let bind_addr = self.ctrl.local_addr()?.ip().to_string() + ":0";
        let listener = TcpListener::bind(bind_addr)?;
        self.data_listener = Some(listener);
        Ok(())
    }
}

    pub fn write_msg(&mut self, msg: &[u8]) -> std::io::Result<usize> {
        self.ctrl.write(msg)
    }
}

fn handle_syst(_args: &str, state: &mut State) -> io::Result<usize>
{
    println!("handle_syst");
    write_msg(MSG_SYST_RESPONSE, state)
}

fn handle_pasv(_args: &str, state: &mut State) -> std::io::Result<usize> {
    println!("handle_pasv");
    state.close_data_port();
    state.open_data_port()?;

    let listener = state.data_listener.as_ref().unwrap();
    let socket = listener.local_addr()?;
    let ip = socket.ip().to_string().replace('.', ",");
    let port = socket.port();
    let response = format!(
        "227 Entering Passive Mode ({},{},{})\r\n",
        ip,
        port / 256,
        port % 256
    );
    state.write_msg(response.as_bytes())
}

fn handle_type(args: &str, state: &mut State) -> std::io::Result<usize> {
    let args = args.trim();
    if args.eq_ignore_ascii_case("A") {
        state.write_msg(b"200 Type set to ASCII.\r\n")
    } else if args.eq_ignore_ascii_case("I") {
        state.write_msg(b"200 Type set to Binary.\r\n")
    } else {
        state.write_msg(b"504 Command not implemented for that parameter.\r\n")
    }
}
fn handle_stor(args: &str, state: &mut State) -> io::Result<usize>
{
    let mut file = std::fs::File::create(args).unwrap();

//    let _ = state.open_data_port();
    write_msg(b"150 Open BINARY mode data connection.\r\n", state)?;
fn handle_stor(args: &str, state: &mut State) -> std::io::Result<usize> {
    let mut file = std::fs::File::create(args)?;

    state.write_msg(b"150 Open BINARY mode data connection.\r\n")?;

    let listener = match &state.data_listener {
        Some(l) => l,
        None => return state.write_msg(b"425 Use PASV first.\r\n"),
    };

    let (mut data_stream, addr) = listener.accept()?;
    println!("connection from {}", addr);

    let result = std::io::copy(&mut data_stream, &mut file)?;
    println!("copied {} bytes", result);

    state.close_data_port();
    state.write_msg(MSG_CLOSING)
}
fn handle_list(_args: &str, state: &mut State) -> io::Result<usize>
{
fn handle_list(_args: &str, state: &mut State) -> std::io::Result<usize> {
    println!("handle_list");

    let listener = match &state.data_listener {
        Some(l) => l,
        None => return state.write_msg(b"425 Use PASV first.\r\n"),
    };

    let (mut data_stream, addr) = listener.accept()?;
    println!("connection from {}", addr);

    data_stream.write_all(b"stuff stuff stuff\r\n\r\n")?;

    state.close_data_port();
    println!("handle_list done");
    state.write_msg(MSG_CLOSING)
}

fn handle_opts(args: &str, state: &mut State) -> std::io::Result<usize> {
    let args = args.trim();
    let response: &[u8] = if args.eq_ignore_ascii_case("UTF8 ON") || args.eq_ignore_ascii_case("UTF8 OFF") {
        b"200 Command OPTS successful.\r\n"
    } else {
        b"501 Syntax error in arguments.\r\n"
    };
    state.write_msg(response)
}

fn handle_connection(stream: TcpStream) -> std::io::Result<()> {
    let mut state = State::new(stream.try_clone()?);
    let mut reader = BufReader::new(stream);
    state.write_msg(MSG_HELLO)?;

    let s = readline(&mut reader)?;
    let (cmd, arg) = parse_command(&s).ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid command"))?;
    if cmd != "USER" {
        return Ok(());
    }
    println!("username={:?}", arg);
    state.write_msg(MSG_USERNAME_OK)?;

    let s = readline(&mut reader)?;
    let (cmd, arg) = parse_command(&s).ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid command"))?;
    if cmd != "PASS" {
        return Ok(());
    }
    state.write_msg(MSG_PASSWORD_OK)?;
    println!("password={:?}", arg);

    let mut map: HashMap<String, fn(&str, &mut State) -> std::io::Result<usize>> = HashMap::new();
    map.insert("SYST".to_string(), handle_syst);
    map.insert("PASV".to_string(), handle_pasv);
    map.insert("TYPE".to_string(), handle_type);
    map.insert("STOR".to_string(), handle_stor);
    map.insert("LIST".to_string(), handle_list);

    let mut state = State::new(stream);
    map.insert("OPTS".to_string(), handle_opts);

    loop {
        let s = match readline(&mut reader) {
            Ok(s) => s,
            Err(_) => break,
        };
        let (cmd, arg) = match parse_command(&s) {
            Some(res) => res,
            None => continue,
        };

        println!("command={:?}", cmd);
        if cmd == "QUIT" {
            let _ = state.write_msg(MSG_BYE);
            break;
        }

        if let Some(func) = map.get(&cmd) {
            let _ = func(&arg, &mut state);
        } else {
            let _ = state.write_msg(b"502 Command not implemented.\r\n");
        }
    }
    println!("connection done");
    Ok(())
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:2121").expect("Could not bind to port 2121");
    println!("FTP server listening on port 2121");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("new connection={:?}", stream);
                thread::spawn(|| {
                    if let Err(e) = handle_connection(stream) {
                        eprintln!("Error handling connection: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command() {
        assert_eq!(parse_command("USER anonymous"), Some(("USER".to_string(), "anonymous".to_string())));
        assert_eq!(parse_command("pass password"), Some(("PASS".to_string(), "password".to_string())));
        assert_eq!(parse_command("QUIT"), Some(("QUIT".to_string(), "".to_string())));
        assert_eq!(parse_command("  LIST -l  "), Some(("LIST".to_string(), "-l".to_string())));
        assert_eq!(parse_command(""), None);
    }

    // ...
}