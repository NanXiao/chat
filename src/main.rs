use std::env;
use std::process;
use std::net::{TcpListener, TcpStream};
use std::str;
use std::thread;
use std::io::{self, BufRead, BufReader, Write};
use std::time::Instant;
use std::error::Error;


const PORT : &'static str = "8001";
const ACK : &'static str = "message received\n";

fn handle_stdin(mut stream: TcpStream) {
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
            Ok(s) => {
                let elapsed = start.elapsed();
                print!("{}", s);
                println!("Roundtrip time is {} us",
                         (elapsed.as_secs() * 1_000_000) + (elapsed.subsec_nanos() / 1_000) as u64);
            }
            Err(e) => {
                println!("str::from_utf8 error: {}", e.description());
                break;
            }
        }
    }
}

fn handle_stream(mut stream: TcpStream) {
    loop {
        {
            let mut buffer = Vec::new();
            let mut reader = BufReader::new(&stream);
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

fn handle_io(stream: TcpStream) {
    match stream.try_clone() {
        Ok(stream_clone) => {
            thread::spawn({move || handle_stdin(stream_clone)});
            handle_stream(stream);
        }
        Err(e) => {
            println!("Clone failed: {}.", e.description());
            process::exit(1);
        }
    }
}

fn get_addr(ip : &str, port : &str) -> String {
    let mut addr = String::new();

    addr.push_str(ip);
    addr.push_str(":");
    addr.push_str(port);

    addr
}

fn client(ip: &str) {
    let addr = get_addr(ip, PORT);
    let peer = addr.clone();

    match TcpStream::connect(addr) {
        Ok(stream) => {
            println!("Connect {} successfully.", peer);
            handle_io(stream);
        }
        Err(e) => {
            println!("Connect {} error: {}.", peer, e.description());
        }
    }
}

fn server() {
    let addr = get_addr("0.0.0.0", PORT);
    let local = addr.clone();

    let listener: TcpListener;
    match TcpListener::bind(addr) {
        Ok(l) => {
            listener = l;
            println!("Bind {} successfully.", local);
        },
        Err(e) => {
            println!("Bind {} error: {}.", local, e.description());
            process::exit(1);
        }
    }

    for stream in listener.incoming() {
        match stream {
            Err(e) => {
                println!("Receive connection failed: {}", e.description());
            }
            Ok(stream) => {
                let peer = stream.peer_addr();
                match peer {
                    Ok(p) => {
                        println!("Client address: {}:{}.", p.ip(), p.port());
                        handle_io(stream);
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