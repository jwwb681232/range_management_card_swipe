use std::thread::{sleep, spawn};
use std::time::Duration;
use chrono::Local;
use ws::{CloseCode, connect};
use redis::Commands;

struct Server {
    ws_sender: ws::Sender,
}

impl ws::Handler for Server {
    fn on_open(&mut self, _shake: ws::Handshake) -> ws::Result<()> {
        Ok(())
    }

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        self.ws_sender.broadcast(msg).unwrap();

        Ok(())
    }
}

fn main() {

    let t1 = spawn(|| {
        let client = redis::Client::open("redis://127.0.0.1:6379").unwrap();
        let mut con = client.get_connection().unwrap();
        loop {
            //todo read file and set file empty
            let _: () = con.publish("card_swipe",Local::now().format("%Y-%m-%d %H:%M:%S").to_string()).unwrap();
            sleep(Duration::from_millis(350));
        }
    });

    let t2 = spawn(|| {
        let mut write_con = redis::Client::open("redis://127.0.0.1:6379").unwrap().get_connection().unwrap();
        let mut write_pubsub = write_con.as_pubsub();
        write_pubsub.subscribe("card_swipe").unwrap();

        loop {
            let msg = write_pubsub.get_message().unwrap();
            let payload : String = msg.get_payload().unwrap();

            ws::connect("ws://127.0.0.1:8085", move|out| {
                out.send(payload.to_owned()).unwrap();
                move |_| {
                    out.close(ws::CloseCode::Normal).unwrap();
                    Ok(())
                }
            }).unwrap();

        }
    });

    ws::listen("0.0.0.0:8085", |ws_sender| Server { ws_sender }).unwrap();

    t1.join();
    t2.join();
}
