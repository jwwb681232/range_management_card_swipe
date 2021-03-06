use std::io::Read;
use std::thread::{sleep, spawn};
use std::time::Duration;
use chrono::Local;
use redis::Commands;
// CQB:         deck1_panel2    19      8085
// 25M Range:   deck1_panel1    30      8087

// 1、替换 Deck1_Panel1 Read From PLC.txt 为 Deck1_Panel2 Read From PLC.txt
// 2、替换 8085 为 8087
// 3、替换 card_swipe_cqb 为 card_swipe_25m

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

    println!("\n25M Range Card Swipe Started On ws://127.0.0.1:8087\n");

    let t1 = spawn(|| {
        let client = redis::Client::open("redis://127.0.0.1:63790").unwrap();
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

            file.read_to_string(&mut contents).unwrap_or_default();

            if contents.len() <=0 {
                pre = "".to_string();
                has_file_notify = true;
                continue;
            }

            let res = contents.split(";").collect::<Vec<&str>>();
            let res = res.get(29).unwrap();

            if &pre != res {
                let _: () = con.publish("card_swipe_25m",format!("{}",res)).unwrap();
                pre = res.to_string();
                println!("[{}]: {}",Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),pre);
            }


            sleep(Duration::from_millis(350));
        }
    });

    let t2 = spawn(|| {
        let mut write_con = redis::Client::open("redis://127.0.0.1:63790").unwrap().get_connection().unwrap();
        let mut write_pubsub = write_con.as_pubsub();
        write_pubsub.subscribe("card_swipe_25m").unwrap();

        loop {
            let msg = write_pubsub.get_message().unwrap();
            let payload : String = msg.get_payload().unwrap();

            ws::connect("ws://127.0.0.1:8087", move|out| {
                out.send(payload.to_owned()).unwrap();
                move |_| {
                    out.close(ws::CloseCode::Normal).unwrap();
                    Ok(())
                }
            }).unwrap();

        }
    });

    ws::listen("0.0.0.0:8087", |ws_sender| Server { ws_sender }).unwrap();

    let _ = t1.join();
    let _ = t2.join();
}
