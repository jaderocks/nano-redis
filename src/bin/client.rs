use bytes::Bytes;
use mini_redis::client;
use tokio::sync::mpsc;

use tokio::sync::oneshot;

#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        val: Bytes,
        resp: Responder<()>,

    }
}

/// 管理任务可以使用该发送端将命令执行的结果传回给发出命令的任务
type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);
    let tx2 = tx.clone();

    let t1 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();

        let cmd = Command::Get {
            key: "hello".to_string(),
            resp: resp_tx,
        };
    
        tx.send(cmd).await.unwrap();
    });

    let t2 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();

        let cmd = Command::Set {
            key: "foo".to_string(),
            val: "bar".into(),
            resp: resp_tx
        };
    `
        tx2.send(cmd).await.unwrap();
    });

    // while let Some(message) = rx.recv().await {
    //     println!("Got = {}", message);
    // }

    let manager = tokio::spawn(async move {
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();

        while let Some(cmd) = rx.recv().await {
            use Command::*;

            match cmd {
                Get {key, resp} => {
                    let res = client.get(&key).await;
                    println!("Get result of {} is {:?}", key, res);
                    // 忽略错误
                    let _ = resp.send(res);
                }

                Set {key, val, resp} => {
                    let res = client.set(&key, val).await;
                    println!("SET result of {} is {:?}", key, res);

                    // 忽略错误
                    let _ =resp.send(res);
                }
            }
        }
    });

    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();
}