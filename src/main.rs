struct Server {
    ws_sender: ws::Sender,
}

impl ws::Handler for Server {
    fn on_open(&mut self, _shake: ws::Handshake) -> ws::Result<()> {
        Ok(())
    }

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        self.ws_sender.broadcast(msg.clone());
        Ok(())
    }
}

fn main() {
    ws::listen("0.0.0.0:8085", |ws_sender| Server { ws_sender}).unwrap();
}
