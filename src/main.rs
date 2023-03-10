use mini_redis::{Connection, Frame};
use tokio::net::{TcpListener, TcpStream};

pub type Error = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move { process(socket).await });
    }
}

async fn process(socket: TcpStream) -> Result<(), Error> {
    use mini_redis::Command::{self, Get, Set};
    use std::collections::HashMap;
    let mut db: HashMap<String, Vec<u8>> = HashMap::new();
    let mut connection = Connection::new(socket);
    while let Some(frame) = connection.read_frame().await? {
        let response = match Command::from_frame(frame).unwrap() {
            Get(cmd) => {
                if let Some(value) = db.get(cmd.key()) {
                    Frame::Bulk(value.clone().into())
                } else {
                    Frame::Null
                }
            }

            Set(cmd) => {
                db.insert(cmd.key().to_string(), cmd.value().to_vec());
                Frame::Simple("OK".to_string())
            }
            cmd => panic!("unimplemented {:?}", cmd),
        };
        connection.write_frame(&response).await?;
    }
    Ok(())
}
