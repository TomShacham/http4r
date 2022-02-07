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

    pub fn get(self, name: String) -> Option<(String, String)> {
        self.pairs.iter()
            .find(|it| it.0 == name)
            .map(|it| it.clone())
    }
}