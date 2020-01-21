use crate::prelude::*;

use std::io::{Read, Write};

use native_tls::TlsStream;
use std::net::TcpStream;

use super::capabilities::Capability;
use super::response::Response;
use super::stream::Stream;

const LIST: &'static str = "LIST\r\n";
//const CAPABILITIES: &'static [u8; 14] = b"CAPABILITIES\r\n";
//const ARTICLE: &'static [u8; 9] = b"ARTICLE\r\n";
//const BODY: &'static [u8; 6] = b"BODY\r\n";
//const DATE: &'static [u8; 6] = b"DATE\r\n";
const HEAD: &'static str = "HEAD\r\n";
const LAST: &'static str = "LAST\r\n";
const QUIT: &'static str = "QUIT\r\n";
//const HELP: &'static [u8; 6] = b"HELP\r\n";
const NEXT: &'static str = "NEXT\r\n";
//const POST: &'static [u8; 6] = b"POST\r\n";
//const STAT: &'static [u8; 6] = b"STAT\r\n";
//const ARTICLE_END : &'static [u8; 3] = b".\r\n";

macro_rules! simple_command_and_check_code {
    ($fnname:ident, $command:expr, $code:expr) => {
        pub fn $fnname (&mut self) -> Result<Response, NNTPError> {
            self.stream.write_all($command)?;

            match self.read_response() {
                Ok(resp) => {
                    if !resp.expected($code) {
                        println!("expected {}, got {}", $code, &resp.response_line[0..3])
                    }
                    Ok(resp)
                },
                Err(e) => {
                    panic!("got {:?}", e);
//                    Err(e)
                }
            }
//            if let Some(ref resp) =  {
//                if
//            }
//            assert!(response.as_ref().unwrap().expected($code));
//            response
        }
    }
}

#[allow(unused_macros)]
macro_rules! utf8_plz {
    ($rest:expr) => {
        std::str::from_utf8(&$rest[..]).unwrap_or("bad utf8 buddy".into())
    };
}

pub struct Client<W: Read + Write> {
    pub stream: Stream<W>,
    pub capabilities: Option<Vec<Capability>>,
}

impl<W: Read + Write> Client<W> {
    pub fn new(stream: Stream<W>) -> Client<W> {
        Client {
            stream,
            capabilities: None,
        }
    }

    pub fn flush_pipeline(&mut self) -> Result<(), NNTPError> {
        self.stream.flush().map_err(|e| e.into())
    }

    pub fn discovery_capabilities(&mut self) -> Result<(), NNTPError> {
        self.stream.write_all("CAPABILITIES\r\n")?;
        let mut response = self.read_response()?;
        assert_eq!(&response.response_line[0..3], "101");

        response.rest = Some(self.stream.read_to_terminal()?);

        use std::str::from_utf8;

        let mut output = [0u8; 8 * 1024];
        let rest = if self.stream.gzip {
            let rest = response.rest.as_ref().unwrap();

            let mut decompressor = flate2::Decompress::new(true);
            let _flat_response = decompressor
                .decompress(&rest[..], &mut output[..], FlushDecompress::None)
                .expect("hello deflation");

            debug!("total out: {}", decompressor.total_out());

            from_utf8(&output[0..decompressor.total_out() as usize])
                .expect("valid utf8 for gzipped capabilities")
        } else {
            from_utf8(&response.rest.as_ref().unwrap()[..]).expect("valid utf8 for capabilities")
        };

        let caps: Vec<Capability> = rest.lines().map(|x| x.into()).collect();

        self.capabilities.replace(caps);

        Ok(())
    }

    pub fn can(&self, ask_cap: Capability) -> bool {
        if let Some(ref caps) = self.capabilities {
            if let Capability::XFEATURE_COMPRESS(ref ask_xfeatures) = ask_cap {
                for possible_cap in caps {
                    if let Capability::XFEATURE_COMPRESS(ref xfeatures) = possible_cap {
                        let set: HashSet<_> = xfeatures.iter().collect();
                        let ask_set: HashSet<_> = ask_xfeatures.iter().collect();
                        return ask_set.is_subset(&set);
                    }
                }
                return false;
            } else {
                caps.contains(&ask_cap)
            }
        } else {
            false
        }
    }

    /// Reads the first line response from the remote server.
    pub fn read_response(&mut self) -> Result<Response, NNTPError> {
        let response_line = self.stream.read_response_line()?;
        Ok(Response::new(response_line, None))
    }

    pub fn authinfo_user(&mut self, user: &str) -> Result<Response, NNTPError> {
        self.stream
            .write_all(&format!("AUTHINFO USER {}\r\n", user)[..])?;

        let response = self.read_response();
        assert!(response.as_ref().unwrap().expected("381"));

        response
    }

    pub fn authinfo_pass(&mut self, pass: &str) -> Result<Response, NNTPError> {
        self.stream
            .write_all(&format!("AUTHINFO PASS {}\r\n", pass)[..])?;

        let response = self.read_response();
        assert!(response.as_ref().unwrap().expected("281"));
        response
    }

    simple_command_and_check_code!(head, HEAD, "205");
    simple_command_and_check_code!(quit, QUIT, "205");
    simple_command_and_check_code!(list, LIST, "205");
    simple_command_and_check_code!(next, NEXT, "223");
    simple_command_and_check_code!(last, LAST, "205");

    /// Selects a newsgroup
    pub fn group(&mut self, group: &str) -> Result<Response, NNTPError> {
        self.stream.write_all(&format!("GROUP {}\r\n", group)[..])?;

        let response = self.read_response();
        assert!(response.as_ref().unwrap().expected("211"));

        response
    }

    /// Lists articles in a group, you probably don't want this
    pub fn listgroup(&mut self) -> Result<Response, NNTPError> {
        self.stream.write_all(&format!("LISTGROUP\r\n")[..])?;

        let response = self.read_response();
        let _rest = self.stream.read_to_terminal()?;
        //        panic!("response: {:#?}/{}", response, rest.len());
        assert!(response.as_ref().unwrap().expected("211"));

        response
    }

    /// Lists articles in a group based on the provided range, you probably don't want this
    pub fn listgroup_range(
        &mut self,
        group: &str,
        thing: std::ops::Range<usize>,
    ) -> Result<Response, NNTPError> {
        let command = format!("LISTGROUP {} {}-{}\r\n", group, thing.start, thing.end);
        self.stream.write_all(&command[..])?;

        let response = self.read_response();
        println!("got response: {}", response.as_ref().unwrap().response_line);
        let _rest = self.stream.read_to_terminal()?;
        //        panic!(
        //            "response: {:#?}\n\n{}",
        //            response,
        //            std::str::from_utf8(&rest[..]).unwrap()
        //        );
        assert!(response.as_ref().unwrap().expected("211"));

        response
    }

    /// Lists articles in a group, you probably don't want this
    pub fn article_by_id(&mut self, id: usize) -> Result<Response, NNTPError> {
        self.article_by_id_pipeline_write(id)?;
        self.article_by_id_pipeline_read()
    }

    /// Lists articles in a group, you probably don't want this
    pub fn article_by_id_pipeline_write(&mut self, id: usize) -> Result<(), NNTPError> {
        self.stream
            .write_all(&format!("ARTICLE {}\r\n", id)[..])
            .map_err(|e| e.into())
    }

    pub fn article_by_id_pipeline_read(&mut self) -> Result<Response, NNTPError> {
        let mut response = self.read_response()?;

        // If it's not a 220, we shouldn't bother reading the rest
        if !response.response_line.starts_with("220") {
            return Ok(response);
        }

        let rest = self.stream.read_to_terminal()?;
        response.rest.replace(rest);

        Ok(response)
    }

    pub fn xfeature_compress_gzip(&mut self) -> Result<Response, NNTPError> {
        self.stream
            .write_all(&format!("XFEATURE COMPRESS GZIP *\r\n")[..])
            .unwrap();

        let mut response = self.read_response()?;

        // If it's not a 220, we shouldn't bother reading the rest
        if !response.response_line.starts_with("220") {
            return Err(NNTPError::UnexpectedCode(
                response.response_line[0..2].to_string(),
            ));
        }

        self.stream.gzip = true;

        let rest = self.stream.read_to_terminal()?;
        response.rest.replace(rest);

        Ok(response)
    }

    /// Retrieves the headers of the article id.
    pub fn head_by_id(&mut self, article_id: usize) -> Result<Response, NNTPError> {
        self.head_by_id_pipeline_write(article_id)?;
        self.head_by_id_read_pipeline()
    }

    pub fn head_by_id_pipeline_write(&mut self, article_id: usize) -> Result<(), NNTPError> {
        self.stream
            .write_all(&format!("HEAD {}\r\n", article_id)[..])
            .map_err(|e| e.into())
    }

    pub fn head_by_range_pipeline_write(
        &mut self,
        articles: std::ops::Range<usize>,
    ) -> Result<(), NNTPError> {
        self.stream
            .write_all(&format!("HEAD {}-{}\r\n", articles.start, articles.end)[..])
            .map_err(|e| e.into())
    }

    pub fn xhdr_by_range_pipeline_write(
        &mut self,
        header_name: String,
        articles: std::ops::Range<usize>,
    ) -> Result<(), NNTPError> {
        let cmd = format!(
            "XHDR {} {}-{}\r\n",
            header_name, articles.start, articles.end
        );
        self.stream.write_all(&cmd[..])
    }

    pub fn xzhdr_by_id_pipeline_write(&mut self, article_id: usize) -> Result<(), NNTPError> {
        self.stream
            .write_all(&format!("XZHDR {}\r\n", article_id)[..])
    }

    pub fn xzhdr_by_range_pipeline_write(
        &mut self,
        articles: std::ops::Range<usize>,
    ) -> Result<(), NNTPError> {
        self.stream
            .write_all(&format!("XZHDR {}-{}\r\n", articles.start, articles.end)[..])
    }

    pub fn head_by_id_read_pipeline(&mut self) -> Result<Response, NNTPError> {
        let mut response = self.read_response()?;

        // If it's not a 100, we shouldn't bother reading the rest
        if !(response.response_line.starts_with("100") || response.response_line.starts_with("221"))
        {
            //            panic!("no me gusta `{}`", response.response_line);
            return Ok(response);
        }

        let rest = self.stream.read_to_terminal_noisey()?;
        response.rest.replace(rest);

        Ok(response)
    }

    pub fn xzhdr_by_id_read_pipeline(&mut self) -> Result<Response, NNTPError> {
        let mut response = self.read_response()?;
        println!("response: {:#?}", response);

        // If it's not a 100, we shouldn't bother reading the rest
        if !(response.response_line.starts_with("100") || response.response_line.starts_with("221"))
        {
            //            panic!("no me gusta `{}`", response.response_line);
            return Ok(response);
        }

        let rest = self.stream.read_to_terminal_noisey()?;
        response.rest.replace(rest);

        Ok(response)
    }

    pub fn xhdr_by_id_read_pipeline(&mut self) -> Result<Response, NNTPError> {
        let mut response = self.read_response()?;
        println!("response: {:#?}", response);

        // If it's not a 100, we shouldn't bother reading the rest
        if !(response.response_line.starts_with("100") || response.response_line.starts_with("221"))
        {
            //            panic!("no me gusta `{}`", response.response_line);
            return Ok(response);
        }

        let rest = self.stream.read_to_terminal_noisey()?;
        response.rest.replace(rest);

        Ok(response)
    }
}

use flate2::FlushDecompress;
use std::collections::HashSet;
use std::fmt::{Debug, Formatter, Result as FmtResult};

impl Debug for Client<TlsStream<TcpStream>> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_struct("Client")
            .field("stream", &self.stream)
            .field("capabilities", &self.capabilities)
            .finish()
    }
}

impl Debug for Client<TcpStream> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_struct("Client")
            .field("stream", &self.stream)
            .field("capabilities", &self.capabilities)
            .finish()
    }
}

impl Client<TcpStream> {
    /// Helper to easily connect to a host
    pub fn connect(host: &str, port: u16) -> Result<Client<TcpStream>, NNTPError> {
        let stream = Stream::connect(host, port)?;

        Ok(Client::new(stream))
    }
}

impl Client<TlsStream<TcpStream>> {
    /// Helper to easily connect to a TLS host
    pub fn connect_tls(
        host: &str,
        port: u16,
        buf_size: usize,
    ) -> Result<Client<TlsStream<TcpStream>>, NNTPError> {
        let stream = Stream::connect_tls(host, port, buf_size)?;

        Ok(Client::new(stream))
    }
}
