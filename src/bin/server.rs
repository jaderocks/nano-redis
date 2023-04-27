use std::{sync::{Arc, Mutex}, collections::HashMap};

use bytes::Bytes;
use mini_redis::{Connection, Frame};
use tokio::net::{TcpListener, TcpStream};

type Db = Arc<Mutex<HashMap<String, Bytes>>>;

#[tokio::main] 
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let db = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        // opt1:  single thread
        // process(socket).await;

        // 将 handle 克隆一份
        let db = db.clone();

        // opt2: multiple thread 
        // generate fork an thread to handle process
        tokio::spawn(async move {
            process(socket,db).await;
        });
    }
}



async fn process(socket: TcpStream, db: Db) {
    use mini_redis::Command::{self, Get, Set};

    let mut connection = Connection::new(socket);

    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                let mut db = db.lock().unwrap();
                db.insert(cmd.key().to_string(), cmd.value().clone());
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                let db = db.lock().unwrap();
                if let Some(value) = db.get(cmd.key()) {
                    Frame::Bulk(value.clone())
                } else {
                    Frame::Null
                }
            }
            cmd => panic!("unimplemented {:?}", cmd),
        };

        connection.write_frame(&response).await.unwrap();
    }
}


async fn process_v1(socket: TcpStream) {
    use mini_redis::Command::{self, Get, Set};
    use std::collections::HashMap;

     // A hashmap is used to store data
     let mut db = HashMap::new();

     // Connection, provided by `mini-redis`, handles parsing frames from
     // the socket
     let mut conn = Connection::new(socket);
 
    // handle one command:
    // if let Some(frame) = conn.read_frame().await.unwrap() {
    //     println!("Got: {:?}", frame);

    //     // reply an error
    //     let response = Frame::Error("Unimplemented".to_string());
    //     conn.write_frame(&response).await.unwrap();
    // }

    // handle multiple command:
    while let Some(frame) = conn.read_frame().await.unwrap() {
        println!("Got: {:?}", frame);

        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                db.insert(cmd.key().to_string(), cmd.value().to_vec());
                Frame::Simple("OK".to_string())
            }  
            Get(cmd) => {
                if let Some(value) = db.get(cmd.key()) {
                       // `Frame::Bulk` expects data to be of type `Bytes`. This
                    // type will be covered later in the tutorial. For now,
                    // `&Vec<u8>` is converted to `Bytes` using `into()`.
                    Frame::Bulk(value.clone().into())
                } else {
                    Frame::Null
                }
            }
            cmd => panic!("unimplemented {:?}", cmd),
        };

        conn.write_frame(&response).await.unwrap();
    }

}