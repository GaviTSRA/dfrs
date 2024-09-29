use std::io::Read;
use std::{io::Write, net::TcpStream};
use base64::prelude::*;
use flate2::write::GzEncoder;
use flate2::Compression;

use crate::config::Config;
use crate::compile::CompiledLine;
use tungstenite::{connect, Message};
use url::Url;

pub fn send(code: Vec<CompiledLine>, config: Config) {
    match config.sending.api {
        crate::config::SendApi::CodeClient => {
            send_codeclient(code, config);
        }
        crate::config::SendApi::Recode => {
            for line in code {
                send_recode(line.code, line.name, config.debug.connection);
            }
        }
    }
}

fn send_recode(code: String, name: String, debug: bool) {
    let data = ("{\"type\": \"template\", \"source\": \"df.rs\", \"data\": \"{\\\"name\\\": \\\"".to_owned() + &name +" \\\",\\\"data\\\":\\\"" + &compress(code) + "\\\"}\"}\n").to_owned();

    if debug {
        println!("{}", data);
    }

    let server_address = "127.0.0.1:31372";
    match TcpStream::connect(server_address) {
        Ok(mut stream) => {
            if debug {
                println!("Connected to server!");
            }
            match stream.write_all(data.as_bytes()) {
                Ok(_) => {
                    if debug {
                        println!("Data sent successfully!");
                        let mut buffer = [0; 2048];
                        match stream.read(&mut buffer) {
                            Ok(bytes_read) => {
                                if bytes_read > 0 {
                                    let response = String::from_utf8_lossy(&buffer[..bytes_read]);
                                    println!("Server response: {:?}", response);
                                } else {
                                    println!("No data received from the server.");
                                }
                            }
                            Err(err) => eprintln!("Error reading from server: {}", err),
                        }
                    }
                }
                Err(err) => eprintln!("Error sending data to server: {}", err),
            }
        }
        Err(err) => eprintln!("Failed to connect to server: {}", err),
    }
}

fn send_codeclient(code: Vec<CompiledLine>, config: Config) {
    //TODO error handling
    let (mut socket, response) = connect(Url::parse("ws://localhost:31375").unwrap()).expect("Can't connect");
    
    if config.debug.connection {
        println!("Connected to server; {:?}", response)
    }
    
    loop {
        let msg = socket.read().expect("Error reading message");
        
        if config.debug.connection {
            println!("Received: {}", msg);
        }

        if msg.to_text().expect("response should be text") == "auth" {
            break;
        }
    }
    
    socket.send(Message::Text("place swap".into())).unwrap();
    for line in code {
        let data = compress(line.code);
        socket.send(Message::Text(format!("place {}", data))).unwrap();
    }
    socket.send(Message::Text("place go".into())).unwrap();

    loop {
        let msg = socket.read().expect("Error reading message");
        
        if config.debug.connection {
            println!("Received: {}", msg);
        }

        if msg.to_text().expect("response should be text") == "place done" {
            break;
        }
    }
}

fn compress(code: String) -> String {
    let mut compressed_data = Vec::new();
    let mut encoder = GzEncoder::new(&mut compressed_data, Compression::default());
    
    match encoder.write_all(code.as_bytes()) {
        Ok(_) => {},
        Err(err) => panic!("{}", err)
    }
    match encoder.finish() {
        Ok(_) => {},
        Err(err) => panic!("{}", err)
    }

    BASE64_STANDARD.encode(compressed_data)
}   