use base64::prelude::*;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::Read;
use std::{io::Write, net::TcpStream};

use crate::compile::CompiledLine;
use crate::config::Config;
use tungstenite::{connect, Message};
use url::Url;

pub fn send(code: Vec<CompiledLine>, config: Config) {
  match config.sending.api {
    crate::config::SendApi::CodeClientGive => {
      send_codeclient_give(code, config.debug.connection);
    }
    crate::config::SendApi::CodeClientPlace => {
      send_codeclient_place(code, config);
    }
    crate::config::SendApi::Print => {
      for line in code {
        println!("{}", compress(line.code))
      }
    }
  }
}

fn send_codeclient_give(code: Vec<CompiledLine>, debug: bool) {
  let (mut socket, response) =
    connect(Url::parse("ws://localhost:31375").unwrap()).expect("Can't connect");

  if debug {
    println!("Connected to server; {:?}", response)
  }

  socket.send(Message::Text("scopes default".into())).unwrap();

  loop {
    let msg = socket.read().expect("Error reading message");

    if debug {
      println!("Received: {}", msg);
    }

    if msg.to_text().expect("response should be text") == "auth" {
      break;
    }
  }

  for line in code {
    let data = "{\"Count\":1b, \"id\":\"minecraft:ender_chest\", \"components\":{\"minecraft:custom_data\":{PublicBukkitValues:{\"hypercube:codetemplatedata\":\'{\"author\":\"Compiled using dfrs\",\"name\":\""
      .to_owned()
    + &line.name
    + "\",\"version\":1,\"code\":\""
    + &compress(line.code)
    + "\"}'}},\"minecraft:custom_name\":'{\"extra\":[{\"bold\":true,\"color\":\"aqua\",\"text\":\""
    + &line.name
    + "\"}],\"text\":\"\"}'}}";

    if debug {
      println!("{}", data);
    }

    socket
      .send(Message::Text(format!("give {}", data)))
      .unwrap();
  }
}

fn send_codeclient_place(code: Vec<CompiledLine>, config: Config) {
  let (mut socket, response) =
    connect(Url::parse("ws://localhost:31375").unwrap()).expect("Can't connect");

  if config.debug.connection {
    println!("Connected to server; {:?}", response)
  }

  socket
    .send(Message::Text("scopes write_code".into()))
    .unwrap();

  loop {
    let msg = socket.read().expect("Error reading message");

    if config.debug.connection {
      println!("Received: {}", msg);
    }

    if msg.to_text().expect("response should be text") == "auth" {
      if config.debug.connection {
        println!("Authed!");
      }
      break;
    }
  }

  socket.send(Message::Text("place swap".into())).unwrap();
  for line in code {
    let data = compress(line.code);
    socket
      .send(Message::Text(format!("place {}", data)))
      .unwrap();
  }
  socket.send(Message::Text("place go".into())).unwrap();
  if config.debug.connection {
    println!("Data sent");
  }

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
    Ok(_) => {}
    Err(err) => panic!("{}", err),
  }
  match encoder.finish() {
    Ok(_) => {}
    Err(err) => panic!("{}", err),
  }

  BASE64_STANDARD.encode(compressed_data)
}
