pub type QueryType = (String, String);
pub struct Query {
    vec: Vec<QueryType>
}
impl Query {
    pub fn empty() -> Query {
        Query { vec: vec!() }
    }

    pub fn from(vec: Vec<(&str, &str)>) -> Query {
        let mut new = Vec::with_capacity(vec.len());
        for q in vec {
            new.push((q.0.to_string(), q.1.to_string()))
        }
        Query { vec: new }
    }

    pub fn get(self, by: &str) -> Option<String> {
        self.vec.iter().find_map(|pair| {
            if pair.clone().0 == by {
                Some(pair.clone().1)
            } else { None }
        })
    }

    pub fn add(self, pair: (&str, &str)) -> Query {
        let mut new = self.vec.clone();
        new.push((pair.0.to_string(), pair.1.to_string()));
        Query { vec: new }
    }
}