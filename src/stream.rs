use std::io::{Read,Write,Error,ErrorKind,Result,BufRead};

use bufstream::BufStream;
use native_tls::TlsStream;

use std::net::TcpStream;

/// Stream to be used for interfacing with a NNTP server.
pub struct Stream<W: Read + Write> {
    stream: BufStream<W>,
}

impl Stream<TcpStream> where TcpStream: Read+Write {
    pub fn connect(host: &str, port: u16) -> Result<Stream<TcpStream>> {
        let mut tcp_stream = TcpStream::connect((host, port))?;

        Ok(Stream::new(BufStream::new(tcp_stream)))
    }
}

impl Stream<TlsStream<TcpStream>> where TlsStream<TcpStream>: Read+Write {
    pub fn connect_tls(host: &str, port: u16) -> Result<Stream<TlsStream<TcpStream>>> {
        let mut tcp_stream = std::net::TcpStream::connect((host, port))?;

        let connector = native_tls::TlsConnector::new().unwrap();
        let stream = connector.connect(host, tcp_stream).map_err(|x| {
            std::io::Error::new(std::io::ErrorKind::Other, "tls failed")
        })?;

        Ok(Stream::new(BufStream::new(stream)))
    }
}

impl<W: Read + Write> Stream<W> {
    pub fn new(stream: BufStream<W>) -> Stream<W> {
        Stream{ stream }
    }

    pub fn flush(&mut self) -> Result<()> {
        self.stream.flush()
    }

    pub fn write_all(&mut self, command: &str) -> Result<()> {
        self.stream.write_all(command.as_bytes())?;
        self.flush()
    }

    /// Reads the first line sent back after issuing a command
    /// Per the RFC, this line is guaranteed to be UTF8 compatible
    pub fn read_response_line(&mut self) -> Result<String> {
        let mut buffer = String::with_capacity(32);
        let debug_buddy = buffer.clone();
        self.stream.read_line(&mut buffer).
            map(|_| buffer).
            map_err(|e| {
                panic!("e: {}\n{:?}", e, debug_buddy);
                Error::new(ErrorKind::Other, "read line failed")
            })
    }

    /// Reads from the buffer through to the terminal ".\r\n"
    pub fn read_to_terminal(&mut self) -> Result<Vec<u8>> {
        let mut bytes_read = 0;
        let mut buffer = Vec::with_capacity(1024*4); // 4kb buffer


        /// Looks for a terminal by comparing the end of the buffer
        /// after every `\n` character. On the terminal `\r\n.\r\n`
        /// it'll actually search based on both of the `\n`. This behavior
        /// will take the minimum from the buffer, leaving pipelined
        /// messages ready for future reads.
        loop {
            bytes_read += self.stream.read_until(b'\n', &mut buffer)?;
            if &buffer[bytes_read-5 .. bytes_read] == b"\r\n.\r\n" {
                break
            }
        }

        let len = buffer.len();
        buffer.truncate(len-5);

        Ok(buffer)
    }
}