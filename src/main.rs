use std::io::Read;
use std::thread::{sleep, spawn};
use std::time::Duration;
use chrono::Local;
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

    println!("\nStarted On ws://127.0.0.1:8085\n");

    let t1 = spawn(|| {
        let client = redis::Client::open("redis://127.0.0.1:6379").unwrap();
        let mut con = client.get_connection().unwrap();

        let file_path = "Deck1_Panel1 Read From PLC.txt";
        let mut has_file_notify = false;
        let mut pre = String::new();
        loop {
            let mut contents = String::new();

            let mut file =  match std::fs::File::open(file_path) {
                Ok(f) => {
                    has_file_notify = false;
                    f
                },
                Err(e) => {
                    if !has_file_notify {
                        println!("[{} {} ]: {}",Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),e,file_path);
                        has_file_notify = true;
                    }
                    sleep(Duration::from_millis(350));
                    continue;
                }
            };

            file.read_to_string(&mut contents).unwrap();

            let res = contents.split(";").collect::<Vec<&str>>();
            let res = res.get(res.len() - 2).unwrap();

            if &pre != res {
                let _: () = con.publish("card_swipe",format!("{}",res)).unwrap();
                pre = res.to_string();
            }


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

    let _ = t1.join();
    let _ = t2.join();
}
