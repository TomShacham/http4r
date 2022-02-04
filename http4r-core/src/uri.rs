use std::fmt;
use regex::Regex;

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Uri<'a> {
    pub scheme: Option<&'a str>,
    pub authority: Option<&'a str>,
    pub path: &'a str,
    pub query: Option<&'a str>,
    pub fragment: Option<&'a str>,
}


#[allow(non_snake_case)]
impl<'a> Uri<'a> {
    pub fn parse(value: &'a str) -> Uri<'a> {
        let RFC3986: Regex = Regex::new("^(?:([^:/?\\#]+):)?(?://([^/?\\#]*))?([^?\\#]*)(?:\\?([^\\#]*))?(?:\\#(.*))?").unwrap();

        let result = RFC3986.captures(value).unwrap();
        Uri {
            scheme: result.get(1).map(|s| s.as_str()),
            authority: result.get(2).map(|s| s.as_str()),
            path: result.get(3).unwrap().as_str(),
            query: result.get(4).map(|s| s.as_str()),
            fragment: result.get(5).map(|s| s.as_str()),
        }
    }

    pub fn with_path(self, path: &'a str) -> Uri<'a> {
        Uri { path, ..self }
    }

    pub fn with_scheme(self, scheme: &'a str) -> Uri<'a> {
        Uri { scheme: Some(scheme), ..self }
    }

    pub fn with_authority(self, authority: &'a str) -> Uri<'a> {
        Uri { authority: Some(authority), ..self }
    }

    pub fn with_query(self, query: &'a str) -> Uri<'a> {
        Uri { query: Some(query), ..self }
    }

    pub fn with_fragment(self, fragment: &'a str) -> Uri<'a> {
        Uri { fragment: Some(fragment), ..self }
    }
}

impl<'a> fmt::Display for Uri<'a> {
    fn fmt(&self, format: &mut fmt::Formatter) -> fmt::Result {
        if let Some(scheme) = self.scheme {
            write!(format, "{}:", scheme)?;
        }
        if let Some(authority) = self.authority {
            write!(format, "//{}", authority)?;
        }
        format.write_str(self.path)?;
        if let Some(query) = self.query {
            write!(format, "?{}", query)?;
        }
        if let Some(fragment) = self.fragment {
            write!(format, "#{}", fragment)?;
        }
        Ok(())
    }
}