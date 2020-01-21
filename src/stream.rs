use crate::prelude::*;

use std::io::{BufRead, Read, Write};

use bufstream::BufStream;
use native_tls::TlsStream;

use std::net::TcpStream;

use pretty_bytes::converter::convert;

/// Stream to be used for interfacing with a NNTP server.
pub struct Stream<W: Read + Write> {
    pub stream: BufStream<W>,
    bytes_read: usize,
    bytes_written: usize,
    started_at: std::time::Instant,
    pub gzip: bool,
}

impl std::fmt::Debug for Stream<TcpStream> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Stream")
            .field("stream", &"TcpStream")
            .field("bytes_read", &convert(self.bytes_read as f64))
            .field("bytes_written", &convert(self.bytes_written as f64))
            .field("started_at", &self.started_at.elapsed())
            .field("gzip", &self.gzip)
            .finish()
    }
}

impl std::fmt::Debug for Stream<TlsStream<TcpStream>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Stream")
            .field("stream", &"TlsStream<TcpStream>")
            .field("bytes_read", &convert(self.bytes_read as f64))
            .field("bytes_written", &convert(self.bytes_written as f64))
            .field("started_at", &self.started_at.elapsed())
            .finish()
    }
}

impl Stream<TcpStream>
where
    TcpStream: Read + Write,
{
    pub fn connect(host: &str, port: u16) -> Result<Stream<TcpStream>, NNTPError> {
        let tcp_stream = TcpStream::connect((host, port))?;

        Ok(Stream::new(BufStream::new(tcp_stream)))
    }
}

impl Stream<TlsStream<TcpStream>>
where
    TlsStream<TcpStream>: Read + Write,
{
    pub fn connect_tls(
        host: &str,
        port: u16,
        buf_size: usize,
    ) -> Result<Stream<TlsStream<TcpStream>>, NNTPError> {
        let tcp_stream = std::net::TcpStream::connect((host, port))?;

        let connector = native_tls::TlsConnector::new().unwrap();
        let stream = connector
            .connect(host, tcp_stream)
            .map_err(|_x| NNTPError::TLSFailed)?;

        Ok(Stream::new(BufStream::with_capacities(
            buf_size, buf_size, stream,
        )))
    }
}

impl<W: Read + Write> Stream<W> {
    pub fn new(stream: BufStream<W>) -> Stream<W> {
        Stream {
            stream,
            bytes_read: 0,
            bytes_written: 0,
            started_at: std::time::Instant::now(),
            gzip: false,
        }
    }

    pub fn flush(&mut self) -> Result<(), NNTPError> {
        self.stream.flush().map_err(|e| e.into())
    }

    pub fn write_all(&mut self, command: &str) -> Result<(), NNTPError> {
        debug!("{}", command.trim());
        let bytes = command.as_bytes();
        self.bytes_written += bytes.len();
        self.stream.write_all(bytes)?;
        self.flush()
    }

    /// Reads the first line sent back after issuing a command
    /// Per the RFC, this line is guaranteed to be UTF8 compatible
    pub fn read_response_line(&mut self) -> Result<String, NNTPError> {
        info!("read response line");
        let mut buffer = String::with_capacity(32);
        let line = self
            .stream
            .read_line(&mut buffer)
            .map(|_| {
                self.bytes_read += buffer.len();
                buffer
            })
            .map_err(|_e| NNTPError::ReadLineFailed);

        //        let mut buf = [0u8; 4 * 1024];
        if self.gzip {
            //            let mut decompressor = flate2::Decompress::new(false);
            //            let line = line.as_ref().unwrap();
            //            info!("decompressing line: {}", line);
            //            let res = decompressor
            //                .decompress(
            //                    &line[0..line.len() - 2].as_bytes(),
            //                    &mut buf[..],
            //                    FlushDecompress::None,
            //                )
            //                .expect("decompress");
            //            if res != Status::Ok {
            //                panic!("wah")
            //            }
            //            debug!("line: {}", line);
            //            let blob =
            //                std::str::from_utf8(&buf[0..decompressor.total_out() as usize]).expect("utf8");
            //            panic!("{}", blob);
        }

        line
    }

    /// Reads from the buffer through to the terminal "\r\n.\r\n"
    pub fn read_to_terminal(&mut self) -> Result<Vec<u8>, NNTPError> {
        let mut bytes_read = 0;
        let mut buffer = Vec::with_capacity(1024 * 4); // 4kb buffer

        // Looks for a terminal by comparing the end of the buffer
        // after every `\n` character. On the terminal `\r\n.\r\n`
        // it'll actually search based on both of the `\n`. This behavior
        // will take the minimum from the buffer, leaving pipelined
        // messages ready for future reads.
        loop {
            let read_in_loop = self.stream.read_until(b'\n', &mut buffer)?;

            bytes_read += read_in_loop;
            self.bytes_read += bytes_read;

            debug!("length: {}, terminator {:?}", buffer.len(), b".\r\n");
            debug!("{:?}", buffer);

            if &buffer[bytes_read - 3..bytes_read] == b".\r\n" {
                debug!("breaking...");
                break;
            }
        }

        let len = buffer.len();
        buffer.truncate(len - 5);

        Ok(buffer)
    }

    /// Reads from the buffer through to the terminal "\r\n.\r\n"
    pub fn read_to_terminal_noisey(&mut self) -> Result<Vec<u8>, NNTPError> {
        let mut bytes_read = 0;
        let mut buffer = Vec::with_capacity(1024 * 4); // 4kb buffer
                                                       //        let mut buffer = vec![0; 1024 * 4]; // INITIALIZED 4kb buffer

        //        loop {
        //            let res = self.stream.read(&mut buffer).expect("noisey read");
        //            if res > 0 {
        //                trace!("buf: {:#?}", &buffer[0..res]);
        //            }
        //            std::thread::sleep(std::time::Duration::from_secs(1));
        //        }

        // Looks for a terminal by comparing the end of the buffer
        // after every `\n` character. On the terminal `\r\n.\r\n`
        // it'll actually search based on both of the `\n`. This behavior
        // will take the minimum from the buffer, leaving pipelined
        // messages ready for future reads.
        loop {
            let read_in_loop = self.stream.read_until(b'\n', &mut buffer)?;

            bytes_read += read_in_loop;
            self.bytes_read += bytes_read;

            //            info!("I'm looking for {:?}", b"\r\n.\r\n");
            //            info!("buf: {:?}", buffer);
            if &buffer[bytes_read - 5..bytes_read] == b"\r\n.\r\n" {
                break;
            }
        }

        let len = buffer.len();
        buffer.truncate(len - 5);

        Ok(buffer)
    }
}
