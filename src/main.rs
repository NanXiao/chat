use std::env;
use std::process;
use std::net::{TcpListener, TcpStream};
use std::str;
use std::thread;
use std::io::{self, BufRead, BufReader, Write};
use std::time::Instant;
use std::error::Error;


const SERVER_PORT : &'static str = "8001";
const CLIENT_PORT : &'static str = "8002";
const ACK : &'static str = "message received\n";

fn handle_send(mut stream: TcpStream) {
    loop {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(n) => {
                if n == 0 {
                    break;
                }
            }
            Err(e) => {
                println!("read_line error: {}", e.description());
                break;
            }
        }

        let start = Instant::now();
        match stream.write_all(input.as_bytes()) {
            Ok(_) => {}
            Err(e) => {
                println!("write_all error: {}", e.description());
                break;
            }
        }

        let mut reader = BufReader::new(&stream);
        let mut buffer: Vec<u8> = Vec::new();
        match reader.read_until(b'\n', &mut buffer) {
            Ok(n) => {
                if n == 0 {
                    break;
                }
            }

            Err(e) => {
                println!("read_until error: {}", e.description());
                break;
            }
        }

        match str::from_utf8(&buffer) {
            Ok(_) => {
                let elapsed = start.elapsed();
                println!("Message is sent successfully, and the roundtrip time is {} us",
                         (elapsed.as_secs() * 1_000_000) + (elapsed.subsec_nanos() / 1_000) as u64);
            }
            Err(e) => {
                println!("str::from_utf8 error: {}", e.description());
                break;
            }
        }
    }
}

fn handle_receive(mut stream: TcpStream) {
    loop {
        {
            let mut buffer = Vec::new();
            let mut reader = BufReader::new(&stream);
            match reader.read_until(b'\n', &mut buffer) {
                Ok(n) => {
                    if n == 0 {
                        break;
                    }
                    match str::from_utf8(&buffer) {
                        Ok(s) => {
                            print!("Receive: {}", s);
                        }
                        Err(e) => {
                            println!("str::from_utf8 error: {}", e.description());
                            break;
                        }
                    }
                }
                Err(e) => {
                    println!("read_until error: {}", e.description());
                    break;
                }
            }
        }

        match stream.write_all(ACK.as_bytes()) {
            Ok(_) => {}
            Err(e) => {
                println!("write_all error: {}", e.description());
                break;
            }
        }
    }
}

fn handle_io(send_stream: TcpStream, receive_stream: TcpStream) {
    thread::spawn({move || handle_send(send_stream)});
    handle_receive(receive_stream);
}

fn get_addr(ip : &str, port : &str) -> String {
    let mut addr = String::new();

    addr.push_str(ip);
    addr.push_str(":");
    addr.push_str(port);

    addr
}

fn client(ip: &str) {
    let listener = create_listener("0.0.0.0", CLIENT_PORT, false);
    let send_stream = connect(ip, SERVER_PORT, true);

    for stream in listener.incoming() {
        match stream {
            Err(e) => {
                println!("Receive connection failed: {}", e.description());
                break;
            }
            Ok(receive_stream) => {
                handle_io(send_stream, receive_stream);
                break;
            }
        }
    }
}

fn connect(ip: &str, port: &str, show: bool) -> TcpStream {
    let addr = get_addr(ip, port);
    let peer = addr.clone();

    match TcpStream::connect(addr) {
        Ok(stream) => {
            if show {
                println!("Connect {} successfully.", peer);
            }
            return stream;
        }
        Err(e) => {
            println!("Connect {} error: {}.", peer, e.description());
            process::exit(1);
        }
    }
}

fn create_listener(ip: &str, port: &str, show: bool) -> TcpListener {
    let addr = get_addr(ip, port);
    let local = addr.clone();

    let listener: TcpListener;
    match TcpListener::bind(addr) {
        Ok(l) => {
            listener = l;
            if show {
                println!("Bind {} successfully.", local);
            }
            return listener;
        },
        Err(e) => {
            println!("Bind {} error: {}.", local, e.description());
            process::exit(1);
        }
    }
}

fn server() {
    let listener = create_listener("0.0.0.0", SERVER_PORT, true);

    for receive_stream in listener.incoming() {
        match receive_stream {
            Err(e) => {
                println!("Receive connection failed: {}", e.description());
            }
            Ok(receive_stream) => {
                let peer = receive_stream.peer_addr();
                match peer {
                    Ok(p) => {
                        println!("Client address: {}:{}.", p.ip(), p.port());
                        handle_io(connect(&(format!("{}", p.ip()))[..], CLIENT_PORT, false),receive_stream);
                    }
                    Err(e) => {
                        println!("Connection error: {}.", e.description());
                    }
                }
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        server();
    } else if args.len() == 2 {
        client(&args[1]);
    } else {
        println!("Usage: server | client IP");
        process::exit(1);
    }
}