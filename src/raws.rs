use std::collections::HashMap;

use serde_json::Value;




pub struct RawTree {
    level : HashMap<String,Raw>,
    subtrees : HashMap<String,RawTree>
}

impl RawTree {
    pub fn new(path : &str) -> RawTree {
        Self { level : HashMap::new(), subtrees : HashMap::new() }
    }
    fn insert(&mut self, raw : Raw, path_remaining : &[String]) {
        if path_remaining.len() == 1 {
            self.level.insert(path_remaining[0].clone(), raw);
        } else {
            let mut t = RawTree { level: HashMap::new(), subtrees:  HashMap::new() };
            t.insert(raw, &path_remaining[0..path_remaining.len()-1]);
            self.subtrees.insert(path_remaining[path_remaining.len() - 1].clone(), t);
        }
    }
}

fn split_path(path : &str) -> Vec<String> {
    let mut v = Vec::new();
    for s in path.split("/"){
        v.push(s.to_owned());
    }
    v
}

pub struct Raw {
    dat : Value,
    path : Vec<String>,
}

impl Raw { 
    fn dat(&self) -> &Value {
        &self.dat
    }
    fn path(&self) -> &Vec<String> {
        &self.path
    }
}
