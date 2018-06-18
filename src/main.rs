extern crate rusqlite;
extern crate url;

use std::env;
use std::path::Path;
use std::net::{TcpListener, TcpStream};
use std::io;
use std::io::{BufRead, BufReader, Write};
use std::thread;
use std::sync::{Arc, Mutex};
use std::str;
use rusqlite::Connection;
use url::percent_encoding::percent_decode;

fn get_command_line() -> (String, Vec<String>) {
    let progname = env::args().next().unwrap();
    let args: Vec<String> = env::args().skip(1).collect();
    (progname, args)
}

fn query_database(db: &mut Connection,
    cmd: &str) -> std::string::String {
    println!("executed cmd: {}", cmd);
    match db.execute(cmd, &[]) {
        Ok(_code) => {
            "Success.".to_string()
        },
        Err(err) => {
            println!("{}", err);
            "Error occurs.".to_string()
        }
    }
}

fn handle_client(stream: TcpStream,
    db: &mut Connection) {
    let mut stream = BufReader::new(stream);

    let mut first_line = String::new();
    if let Err(err) = stream.read_line(&mut first_line) {
        panic!("Error {}", err);
    }

    let mut params = first_line.split_whitespace();
    let method = params.next();
    let path = params.next();

    match (method, path) {
        (Some("GET"), Some(file_path)) => {
            println!("GET {}", file_path);

            if file_path.starts_with("/?") {
                let query_str = &file_path[2..];
                let query_str_decoded = percent_decode(query_str.as_bytes()).decode_utf8().unwrap();
                let str_splitted = query_str_decoded.split('=');
                let sql_cmd = str_splitted.last().unwrap();
                let result = query_database(db, sql_cmd);
                send_message_to_client(stream.get_mut(), &result);
            }
        },
        _ => panic!("Failed to parse!"),
    }
}

fn send_message_to_client(stream: &mut TcpStream,
    message: &String) {
    writeln!(stream, "HTTP/1.1 200 OK").unwrap();
    writeln!(stream, "Content-Type: text/html; charset=UTF-8").unwrap();
    writeln!(stream, "Content-Length: {}", message.len()).unwrap();
    writeln!(stream).unwrap();
    writeln!(stream, "{}", message).unwrap();
}

fn main() -> io::Result<()> {
    let (progname, args) = get_command_line();
    if args.is_empty() {
        eprintln!("");
        eprintln!("Usage: {} sample.db", progname);
        panic!("");
    }

    let listener = TcpListener::bind("127.0.0.1:12345").unwrap();

    let path = Path::new(&args[0]);
    let db = Arc::new(Mutex::new(Connection::open(path).unwrap()));

    for stream in listener.incoming() {
        let db = db.clone();
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    let mut db = db.lock().unwrap();
                    handle_client(stream, &mut db);
                });
            }
            Err(_) => { panic!("connection failed.") }
        }
    }

    Ok(())
}
