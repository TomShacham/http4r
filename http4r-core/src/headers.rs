pub struct Headers {
    pub vec: HeadersType,
}

pub type HeaderType = (String, String);
pub type HeadersType = Vec<HeaderType>;

impl Headers {
    pub fn empty() -> Headers {
        Headers { vec: vec!() }
    }

    pub fn from(pairs: Vec<(&str, &str)>) -> Headers {
        let mut headers = Headers { vec: vec!() };
        for pair in pairs {
            headers = headers.add(pair)
        }
        headers
    }

    pub fn add(&self, header: (&str, &str)) -> Headers {
        let mut new: HeadersType = vec!();
        let mut exists = false;
        for existing in &self.vec {
            if existing.0 == header.0 {
                let string = existing.clone().1.to_string() + ", " + header.1;
                new.push((existing.clone().0, string.clone()));
                exists = true
            } else {
                new.push(existing.clone())
            }
        }
        if !exists {
            new.push((header.0.to_string(), header.1.to_string()))
        }
        Headers { vec: new }
    }

    pub fn replace(&self, replacing: (&str, &str)) -> Headers {
        let mut new: HeadersType = vec!();
        let mut exists = false;
        for existing in &self.vec {
            if existing.0 == replacing.0 {
                new.push((existing.clone().0, replacing.1.to_string()));
                exists = true
            } else {
                new.push((existing.0.to_string(), existing.1.to_string()))
            }
        }
        if !exists {
            new.push((replacing.0.to_string(), replacing.1.to_string()));
        }
        Headers { vec: new }
    }

    pub fn get(&self, name: &str) -> Option<String> {
        for header in &self.vec {
            if header.0.to_lowercase() == name.to_lowercase() {
                return Some(header.1.to_string());
            }
        }
        None
    }

    pub fn content_length_header(&self) -> Option<usize> {
        self.get("Content-Length").map(|x| { x.parse().unwrap() })
    }

    pub fn parse_from(header_string: &str) -> Headers {
        if header_string.is_empty() {
            return Headers::empty();
        }
        header_string.split("\r\n").fold(Headers::empty(), |acc, pair| {
            let pair = pair.split(": ").collect::<Vec<&str>>();
            acc.add((pair[0], pair[1]))
        })
    }

    pub fn to_wire_string(&self) -> String {
        self.vec.iter().map(|h| {
            let clone = h.clone();
            clone.0.to_owned() + ": " + clone.1.as_str()
        }).collect::<Vec<String>>()
            .join("\r\n")
    }

    pub fn js_headers_from_string(str: &str) -> Headers {
        str.split("; ").fold(Headers::empty(), |acc: Headers, next: &str| {
            let mut split = next.split(": ");
            acc.add((split.next().unwrap(), split.next().unwrap()))
        })
    }

    pub fn js_headers_to_string(headers: &HeadersType) -> String {
        headers.iter().map(|h| {
            let clone = h.clone();
            clone.0 + ": " + clone.1.as_str()
        }).collect::<Vec<String>>()
            .join("; ")
    }

}

