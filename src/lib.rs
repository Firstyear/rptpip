
#[macro_use]
extern crate nom;
extern crate byteorder;

use byteorder::{ByteOrder, LittleEndian};

use std::mem;
use std::io::{self, Read, Write};

enum PTPServerState {
    Initial,
    HandshakeSize,
    HandshakeType,
    HandshakeData,
    Connected,
    Size,
    Type,
    Data,
    Disconnected,
}

pub struct PTPServer {
    state: PTPServerState,
}

impl PTPServer {
    pub fn new() -> Self {
        PTPServer{
            state: PTPServerState::Initial,
        }
    }
}

impl Write for PTPServer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // Check what the request was, then store the state.
        self.state = match self.state {
            PTPServerState::Initial => {
                PTPServerState::HandshakeSize
            }
            _ => {
                PTPServerState::Disconnected
            }
        };

        Ok(0)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Read for PTPServer {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut rlen: io::Result<usize> = Ok(0);
        self.state = match self.state {
            PTPServerState::HandshakeSize => {
                println!("Sending Handshake Size");
                let response: [u8; 4] = [0x0c, 00, 00, 00];
                let len = response.len();
                buf[..len].clone_from_slice(&response);
                rlen = Ok(len);
                PTPServerState::HandshakeType
            }
            PTPServerState::HandshakeType => {
                println!("Sending Handshake Type");
                let response: [u8; 4] = [0x05, 00, 00, 00];
                let len = response.len();
                buf[..len].clone_from_slice(&response);
                rlen = Ok(len);
                PTPServerState::HandshakeData
            }
            PTPServerState::HandshakeData => {
                println!("Sending Handshake Data");
                let response: [u8; 4] = [0x20, 0x1e, 00, 00];
                let len = response.len();
                buf[..len].clone_from_slice(&response);
                rlen = Ok(len);
                PTPServerState::Connected
            }
            _ => {
                PTPServerState::Disconnected
            }
        };
        // Based on the state return an appropriate response.
        rlen
    }
}

#[derive(Debug)]
pub enum PTPError {
    UnknownError,
    ResponseShort,
    InvalidResponseSize,
    InvalidResponseState,
    HandshakeError,
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u32)]
pub enum PTPResponseCode {
    Unknown = 0x0,
    Success = 0x00002019,
    DeviceBusy = 0x0000201e,
}

impl PTPResponseCode {
    fn new(code: u32) -> Self {
        match code {
            0x00002019 => PTPResponseCode::Success,
            0x0000201e => PTPResponseCode::DeviceBusy,
            _ => PTPResponseCode::Unknown,
        }
    }
}

#[derive(Debug)]
pub enum PTPResult {
    Unknown {},
    UintResult4 {
        result: PTPResponseCode,
    },
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
    pub fn new(transport: T) -> Result<Self, PTPError> {
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

    fn recieve_response(&mut self) -> Result<PTPResult, PTPError> {

        let mut raw_size = [0u8; 4];
        let mut raw_type = [0u8; 4];

        self.transport.read(&mut raw_size);
        self.transport.read(&mut raw_type);

        let rsize: u32 = LittleEndian::read_u32(&raw_size);
        let rtype: u32 = LittleEndian::read_u32(&raw_type);

        // Read the first 8 bytes to get the length and struct type.


        // Now that we know the remaining length, read in the rest - 8.
        if rsize <= 8 {
            return Err(PTPError::ResponseShort);
        }

        let rem_data_size: usize = (rsize - 8) as usize;
        println!("rsize {}, rtype {}, data {} bytes", rsize, rtype, rem_data_size);
        let mut data: Vec<u8> = vec![0; rem_data_size];
        self.transport.read(data.as_mut_slice());

        // println!("raw_data: {:?}", data);

        match rtype {
            0x05 => {
                if rem_data_size != 4 {
                    Err(PTPError::InvalidResponseSize)
                }else {
                    let response_code = LittleEndian::read_u32(data.as_slice());
                    println!("responsecode {}", response_code);

                    Ok(PTPResult::UintResult4 {
                        // Could convert this later?
                        result: PTPResponseCode::new(response_code)
                    })
                }
            }
            _ => {
                Err(PTPError::UnknownError)
            }
        }
    }

    fn handshake(&mut self) -> Result<PTPResult, PTPError> {
        let hs = PTPHandshake::new("Rust PTP/IP");
        let req = hs.to_u8();

        println!("Sending request");
        // Send it to the transport:
        self.transport.write(req.as_slice());
        self.transport.flush();
        // Decode our response and send it back,
        let response = self.recieve_response();

        println!("{:?}", response);

        match response {
            Ok(PTPResult::UintResult4 { result }) => {
                // Ok
                if result != PTPResponseCode::Success {
                    return Err(PTPError::HandshakeError);
                }
            }
            _ => return Err(PTPError::InvalidResponseState),
        };

        response
    }

    fn disconnect(&mut self) {
        // Send the disconnect - 0x00 00 00 00, 0xff ff ff ff
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


