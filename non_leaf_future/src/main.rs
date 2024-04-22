use tokio::io::AsyncWriteExt;

fn main() {
    // This is a non leaf future because it does not represent a resource.
    let non_leaf = async {
        let mut stream = tokio::net::TcpStream::connect("127.0.0.1:3000")
            .await
            .unwrap();
        println!("connected");

        stream.write_all(b"hello world").await.unwrap();
        println!("message sent");
    };
}
