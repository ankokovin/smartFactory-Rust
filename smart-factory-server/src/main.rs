use smart_factory_environment::greet_message;

use tokio::{net::TcpListener, io::{BufReader, AsyncWriteExt, AsyncBufReadExt}};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("localhost:8080").await.unwrap();
    
    let (mut socket, _addr) = listener.accept().await.unwrap();

    let (reader, mut writer) = socket.split();

    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        let _bytes_read = reader.read_line(&mut line).await.unwrap();

        let response_line = greet_message(line.as_str());
        writer.write_all(response_line.as_bytes()).await.unwrap();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}