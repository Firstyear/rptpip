
#[macro_use]
extern crate nom;

use std::mem;
use std::io::{self, Read, Write};

pub struct PTPServer {
}

impl PTPServer {
    pub fn new() -> Self {
        PTPServer{}
    }
}

impl Write for PTPServer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // Check what the request was, then store the state.
        Ok(0)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Read for PTPServer {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // Based on the state return an appropriate response.
        let response: [u8; 12] = [00, 00, 00, 0x0c, 00, 00, 00, 0x05, 00, 00, 0x20, 0x1e];
        let len = response.len();
        buf[..len].clone_from_slice(&response);
        Ok(len)
    }
}

pub enum PTPResult {
    UnknownError
}

// Need a proto

#[derive(Debug,PartialEq)]
#[repr(packed)]
struct PTPHandshake {
    name: String,
}

impl PTPHandshake {
    fn new(name: &str) -> Self {
        PTPHandshake {
            name: String::from(name),
        }
    }

        /*
00000000  52 00 00 00 01 00 00 00  f2 e4 53 8f ad a5 48 5d R....... ..S...H]
00000010  87 b2 7f 0b d3 d5 de d0  03 5f a8 c0 69 00 50 00 ........ ._..i.P.
00000020  68 00 6f 00 6e 00 65 00  2d 00 36 00 34 00 35 00 h.o.n.e. -.6.4.5.
00000030  31 00 2d 00 32 00 30 00  31 00 37 00 2e 00 30 00 1.-.2.0. 1.7...0.
00000040  39 00 32 00 33 00 00 00  00 00 00 00 00 00 00 00 9.2.3... ........
00000050  00 00                                            ..
    00000000  0c 00 00 00 05 00 00 00  1e 20 00 00             ........ . ..
00000052  08 00 00 00 ff ff ff ff                          ........


         */

    fn to_u8(&self) -> Vec<u8> {
        // Build a request. Looking at the packet cap from the xt10, I can see:
        //  52 00 00 00 01 00 00 00
        //  But this looks like it's little endian in a bigendian cap. So I think
        // it's:
        //  00 00 00 52
        //  00 00 00 01
        /* Allocate a vec of size */
        let mut result: Vec<u8> = Vec::with_capacity(0x52);
        /* First push our msg length */
        result.push(0x52);
        result.push(0x00);
        result.push(0x00);
        result.push(0x00);
        /* Now the command type */
        result.push(0x01);
        result.push(0x00);
        result.push(0x00);
        result.push(0x00);
        /* Now we push the header */
        result.extend_from_slice(&[
            0xf2, 0xe4, 0x53, 0x8f,
            0xad, 0xa5, 0x48, 0x5d,
            0x87, 0xb2, 0x7f, 0x0b,
            0xd3, 0xd5, 0xde, 0xd0,
            0x03, 0x5f, 0xa8, 0xc0,
        ]);
        /*
         * Now for each char in the name push char + 00
         */
        let mut namecap = 27;
        let name_bytes = self.name.as_bytes();
        for c in name_bytes {
            result.push(*c);
            result.push(0);
            namecap -= 1;
        }
        /*
         * Fill the remainder with 00
         */
        while namecap > 0 {
            result.push(0);
            result.push(0);
            namecap -= 1;
        }

        /* Now return!*/
        result
    }
}


pub struct PTPClient<T: Read + Write> {
    transport: T
}

impl <T> PTPClient<T>
    where T: Read + Write
{
    pub fn new(transport: T) -> Result<Self, PTPResult> {
        let mut inner = PTPClient {
            transport: transport
        };

        // Now conduct a handshake
        match inner.handshake() {
            Ok(_) => {
                Ok(inner)
            }
            Err(e) => {
                Err(e)
            }
        }
    }

    fn handshake(&mut self) -> Result<(), PTPResult> {
        let hs = PTPHandshake::new("Rust PTP/IP");
        let req = hs.to_u8();

        println!("Sending request");


        // Send it to the transport:
        self.transport.write(req.as_slice());
        self.transport.flush();
        let mut response = [0u8; 128];
        self.transport.read(&mut response);

        // Now decode the response to something?

        // Read the first bytes to get the length.

        for i in 0..4 {
            println!("{}", response[i]);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{PTPClient, PTPServer};

    #[test]
    fn test_handshake() {
        // Create the server
        let mut server = PTPServer::new();
        // Give the server to the client
        let client = PTPClient::new(server);
        assert!(client.is_ok());
    }

}


