fn main() {
    // A leaf future represents a resource.
    let mut stream = tokio::net::TcpStream::connect("127.0.0.1:3000");
}
