pub type QueryType = (String, String);
pub struct Query {
    pub vec: Vec<QueryType>
}

impl From<&str> for Query {
    fn from(str: &str) -> Self {
        let mut query = Query::empty();
        str.split("&").for_each(|q| {
            let pair = q.split("=").collect::<Vec<&str>>();
            let [key, value] = [pair[0], pair[1]];
            query = query.add((key, value));
        });
        query
    }
}

impl From<Vec<(&str, &str)>> for Query {
    fn from(vec: Vec<(&str, &str)>) -> Self {
        let mut new = Vec::with_capacity(vec.len());
        for q in vec {
            new.push((q.0.to_string(), q.1.to_string()))
        }
        Query { vec: new }
    }
}

impl Query {
    pub fn empty() -> Query {
        Query { vec: vec!() }
    }

    pub fn get(self, by: &str) -> Option<String> {
        self.vec.iter().find_map(|pair| {
            if pair.clone().0 == by {
                Some(pair.clone().1)
            } else { None }
        })
    }

    pub fn get_all(&self, p0: &str) -> Vec<QueryType> {
        self.vec.iter().filter(|q|{
            q.clone().0 == p0
        }).map(|t| (t.clone().0, t.clone().1))
            .collect::<Vec<QueryType>>()
    }

    pub fn add(&self, pair: (&str, &str)) -> Query {
        let mut new = vec!();
        for q in &self.vec {
            new.push(q.clone())
        }
        new.push((pair.0.to_string(), pair.1.to_string()));
        Query { vec: new }
    }

    pub fn replace(&self, pair: (&str, &str)) -> Query {
        let mut new = vec!();
        let mut seen = false;
        for q in &self.vec {
            if q.0 == pair.0 && seen == false {
                new.push((pair.0.to_string(), pair.1.to_string()));
                seen = true
            }
            if q.0 != pair.0 {
                new.push(q.clone())
            }
        }
        if seen == false {
            new.push((pair.0.to_string(), pair.1.to_string()));
        }
        Query { vec: new }
    }
}