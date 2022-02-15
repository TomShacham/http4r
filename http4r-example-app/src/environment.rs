use std::env::Vars;

pub struct Environment {
    pairs: Vec<(String, String)>
}
impl Environment {
    pub fn from(vars: Vars) -> Environment {
        let mut vec = Vec::new();
        for pair in vars {
            vec.push(pair);
        }
        Environment {
            pairs: vec
        }
    }

    pub fn with(self, pairs: Vec<(&str, &str)>) -> Environment {
        let mut vec = Vec::new();
        for pair in pairs {
            vec.push((pair.0.to_string(), pair.1.to_string()));
        }
        for pair in self.pairs {
            if !vec.contains(&pair) {
                vec.push(pair);
            }
        }
        Environment {
            pairs: vec
        }
    }

    pub fn get(&self, name: &str) -> Option<String> {
        self.pairs.iter()
            .find(|it| it.0 == name)
            .map(|it| it.clone().1)
    }

    pub fn copy(&self) -> Environment {
        let mut v = Vec::new();
        for pair in &self.pairs {
            v.push(pair.clone())
        }
        Environment {
            pairs: v
        }
    }

    pub fn empty() -> Environment {
        Environment {
            pairs: vec!()
        }
    }

}