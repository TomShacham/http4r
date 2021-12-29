use std::fmt;
use lazy_static::lazy_static;
use regex::Regex;

#[derive(PartialEq, Debug)]
pub struct Uri<'a> {
    pub scheme: Option<&'a str>,
    pub authority: Option<&'a str>,
    pub path: &'a str,
    pub query: Option<&'a str>,
    pub fragment: Option<&'a str>,
}

impl<'a> Uri<'a> {
    pub fn parse(value: &'a str) -> Uri<'a> {
        lazy_static! {
            static ref RFC3986: Regex = Regex::new("^(?:([^:/?\\#]+):)?(?://([^/?\\#]*))?([^?\\#]*)(?:\\?([^\\#]*))?(?:\\#(.*))?").unwrap();
        }

        let result = RFC3986.captures(value).unwrap();
        Uri {
            scheme: result.get(1).map(|s| s.as_str()),
            authority: result.get(2).map(|s| s.as_str()),
            path: result.get(3).unwrap().as_str(),
            query: result.get(4).map(|s| s.as_str()),
            fragment: result.get(5).map(|s| s.as_str()),
        }
    }

    pub fn with_scheme(self, scheme: &'a str) -> Uri<'a> {
        Uri {
            scheme: Some(scheme),
            ..self
        }
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