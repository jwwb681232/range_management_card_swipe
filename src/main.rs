use std::sync::{Mutex, Arc, mpsc::channel};
use std::thread::spawn;

struct Server {
    ws_sender: ws::Sender,
    tx: Arc<Mutex<std::sync::mpsc::Sender<String>>>,
}

impl ws::Handler for Server {
    fn on_open(&mut self, shake: ws::Handshake) -> ws::Result<()> {
        if let Some(ip_addr) = shake.remote_addr()? {
            println!("Connection opened from {}", ip_addr)
        } else {
            println!("Unable to obtain client's IP address")
        }
        Ok(())
    }

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        self.ws_sender.broadcast(msg.clone());
        self.tx.lock().unwrap().send(msg.to_string());
        Ok(())
    }
}

fn main() {
    let (tx, rx) = channel();

    let x = Arc::new(Mutex::new(tx));

    spawn(move ||{
        loop {
            let received = rx.recv().unwrap();
            println!("Got: {}", received);
        }
    });

    ws::listen("0.0.0.0:8085", |ws_sender| Server { ws_sender, tx: x.clone()}).unwrap();
}
