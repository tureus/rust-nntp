use std::fmt::{Debug, Formatter};
use std::io::{Error, ErrorKind, Result};

pub struct Response {
    pub response_line: String,
    pub rest: Option<Vec<u8>>,
}

impl Debug for Response {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let rest = if let Some(ref rest) = self.rest {
            if rest.len() >= 100 {
                let leftover = rest.len() - 100;
                let trunc = std::str::from_utf8(&rest[0..100]).unwrap();
                Some(format!("{}... ({} bytes truncated)", trunc, leftover))
            } else {
                Some(
                    std::str::from_utf8(&rest[0..rest.len()])
                        .unwrap()
                        .to_string(),
                )
            }
        } else {
            None
        };

        f.debug_struct("Response")
            .field("response_line", &self.response_line)
            .field("rest", &rest)
            .finish()
    }
}

const SUCCESS_CODES: [&'static str; 1] = ["221"];

impl Response {
    pub fn new(response_line: String, rest: Option<Vec<u8>>) -> Response {
        Response {
            response_line,
            rest,
        }
    }

    pub fn expected(&self, expected: &str) -> bool {
        self.response_line.starts_with(expected)
    }

    pub fn success(&self) -> bool {
        SUCCESS_CODES
            .iter()
            .any(|&x| self.response_line.starts_with(x))
    }

    /// After issuing a `GROUP $NAME` command you will get a response with three numbers,
    /// this helper will parse that line.
    ///
    /// For reference, these numbers mean
    ///
    /// (est. number of messages, reported low water mark, reported high water mark)
    ///
    pub fn parse_group_stats(&self) -> Result<(usize, usize, usize)> {
        use std::str::FromStr;
        let mut parts = self.response_line.split(" ");
        parts.next().unwrap();
        let res = (
            FromStr::from_str(parts.next().expect("failed on first"))
                .expect("failed to parse first"),
            FromStr::from_str(parts.next().expect("failed on second"))
                .expect("failed to parse second"),
            FromStr::from_str(parts.next().expect("failed on third"))
                .expect("failed to parse third"),
        );
        Ok(res)
    }

    pub fn parse_article_response(&self) -> Result<(&str, &str, &str)> {
        let parts: Vec<_> = self.response_line.split(" ").collect();
        if parts.len() != 3 {
            Err(Error::new(
                ErrorKind::Other,
                "article response did not have 3 parts",
            ))
        } else {
            Ok((parts[0], parts[1], parts[2]))
        }
    }

    pub fn parse_header(&self) -> Result<Vec<(&[u8], &[u8])>> {
        let subbuf = &self.rest.as_ref().unwrap()[0..100];
        panic!("subbuf: {:?}", subbuf);
        //        unimplemented!()
    }
}
