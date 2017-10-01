
extern crate rptpip;
use std::net::TcpStream;

use rptpip::{PTPClient, PTPResult};

fn main() {
    // Connect to the camera.
    let mut server = TcpStream::connect("192.168.0.1:55740").unwrap();
    // Give the server to the client
    let client = PTPClient::new(server);
    assert!(client.is_ok());


}
