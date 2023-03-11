use bytes::Bytes;
use mini_redis::client;
use rand::Rng;
use tokio::sync::{mpsc, oneshot};

type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;
#[derive(Debug)]
enum Command {
    Get {
        key: String,
        responder: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        val: Bytes,
        responder: Responder<()>,
    },
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);
    let tx2 = tx.clone();

    let t1 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Get {
            key: "foo".to_string(),
            responder: resp_tx,
        };
        tx.send(cmd).await.unwrap();

        let res = resp_rx.await;
        println!("GET response = {:?}", res);
    });
    let mut rng = rand::thread_rng();

    let val = rng.gen::<i32>().to_string();

    let t2 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();

        let cmd = Command::Set {
            key: "foo".to_string(),
            val: val.into(),
            responder: resp_tx,
        };
        tx2.send(cmd).await.unwrap();
        let res = resp_rx.await;
        println!("SET response = {:?}", res);
    });
    let manager = tokio::spawn(async move {
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();
        while let Some(cmd) = rx.recv().await {
            use Command::*;

            match cmd {
                Get { key, responder } => {
                    let response = client.get(&key).await;
                    responder.send(response).unwrap();
                }
                Set {
                    key,
                    val,
                    responder,
                } => {
                    let response = client.set(&key, val).await;
                    responder.send(response).unwrap();
                }
            }
        }
    });

    _ = tokio::join!(t1, t2, manager,);
}
