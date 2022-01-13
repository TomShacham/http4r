
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
            if existing.0.to_lowercase() == header.0.to_lowercase() {
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

    pub fn add_all(&self, headers: Headers) -> Headers {
        let mut new = Headers::empty();
        self.vec.iter().for_each(|x| {
            new = headers.add((x.0.as_str(), x.1.as_str()))
        });
        new
    }

    pub fn replace(&self, replacing: (&str, &str)) -> Headers {
        let mut new: HeadersType = vec!();
        let mut exists = false;
        for existing in &self.vec {
            if existing.0.to_lowercase() == replacing.0.to_lowercase() {
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

    pub fn remove(&self, name: &str) -> Headers {
        let mut new: HeadersType = vec!();
        for existing in &self.vec {
            if existing.0.to_lowercase() == name.to_lowercase() {
            } else {
                new.push((existing.0.to_string(), existing.1.to_string()))
            }
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

    pub fn filter(&self, names: Vec<&str>) -> Headers {
        let mut vec = vec!();
        for name in names {
            if let Some(value) = self.get(name) {
                vec.push((name.to_string(), value) as HeaderType);
            }
        }
        Headers { vec }
    }

    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    pub fn has(&self, header_name: &str) -> bool {
        return self.get(header_name).is_some();
    }

    pub fn content_length_header(&self) -> Option<Result<usize, String>> {
        let value = self.get("Content-Length");
        let value = Self::parse_or_else_value(value);
        match value {
            Some(Err(ref header_value)) => {
                let split = header_value.split(", ").map(|it| it.to_string()).collect::<Vec<String>>();
                let first = split.first().map(|it| it.to_string());
                if split.iter().all(|v| v == first.as_ref().unwrap()) {
                    Self::parse_or_else_value(first)
                } else {
                    value.clone()
                }
            }
            _ => value
        }
    }

    fn parse_or_else_value(value: Option<String>) -> Option<Result<usize, String>> {
        value.as_ref().map(|x| x.parse::<usize>())
            .map(|r| r.map_err(|_| value.unwrap()))
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

