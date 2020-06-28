//# An API to interact with the blockchain network

use actix::prelude::*;
use log::*;
use std::net::TcpListener;
use tungstenite::{accept, Message};

use crate::util::types::Bytes;

pub struct Server {
    listener: TcpListener,
    connections: Vec<tungstenite::protocol::WebSocket<std::net::TcpStream>>,
}

impl Server {
    pub fn init() -> Self {
        let listener = TcpListener::bind("127.0.0.1:3012").unwrap();

        Server {
            listener,
            connections: vec![],
        }
    }

    pub fn listen(&mut self) {
        println!(
            "The TCP server is listening for incoming web socket connections on 127.0.0.1:3012"
        );

        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    let socket = accept(stream).unwrap();
                    self.connections.push(socket);
                    break;
                }
                Err(e) => error!("Error accepting stream: {}", e),
            };
        }
    }

    pub fn broadcast(&mut self, msg: String) {
        for socket in self.connections.iter_mut() {
            let msg = Message::Text(msg.clone());
            let result = socket.write_message(msg);

            match result {
                Ok(_) => println!("Broadcasted to socket {:?}", socket),
                Err(e) => println!("Error {:?} broadcasting to socket {:?}", e, socket),
            };
        }
    }
}

impl Actor for Server {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ServerMessage(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct ServerCommand(pub &'static str);

impl Handler<ServerMessage> for Server {
    type Result = ();

    fn handle(&mut self, msg: ServerMessage, _ctx: &mut Context<Self>) -> Self::Result {
        self.broadcast(msg.0);
        ()
    }
}

impl Handler<ServerCommand> for Server {
    type Result = ();

    fn handle(&mut self, msg: ServerCommand, _ctx: &mut Context<Self>) -> Self::Result {
        match msg.0 {
            "Listen" => self.listen(),
            &_ => (),
        };

        ()
    }
}
