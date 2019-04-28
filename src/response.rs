use std::io::{ Result, Error, ErrorKind };

#[derive(Debug)]
pub struct Response {
    pub response_line: String,
    pub rest: Option<Vec<u8>>
}

impl Response {
    pub fn new(response_line: String, rest: Option<Vec<u8>>) -> Response {
        Response{ response_line, rest }
    }

    pub fn expected(&self, expected: &str) -> bool {
        self.response_line.starts_with(expected)
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
            FromStr::from_str(parts.next().expect("failed on first")).expect("failed to parse first"),
            FromStr::from_str(parts.next().expect("failed on second")).expect("failed to parse second"),
            FromStr::from_str(parts.next().expect("failed on third")).expect("failed to parse third"),
        );
        Ok(res)
    }

    pub fn parse_article_response(&self) -> Result<(&str, &str, &str)> {
        let mut parts : Vec<_> = self.response_line.split(" ").collect();
        if parts.len() != 3 {
            Err(Error::new(ErrorKind::Other, "article response did not have 3 parts"))
        } else {
            Ok((parts[0], parts[1], parts[2]))
        }
    }
}