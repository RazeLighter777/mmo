use std::{collections::HashMap, io::Read};

use serde::de::DeserializeOwned;
use serde_json::Value;
use std::fs;

#[derive(Debug)]
pub struct RawTree {
    level: HashMap<String, Raw>,
    subtrees: HashMap<String, RawTree>,
}

impl RawTree {
    pub fn new_empty() -> RawTree {
        Self {
            level: HashMap::new(),
            subtrees: HashMap::new(),
        }
    }
    pub fn new(path: &str) -> RawTree {
        println!("The raw path is {}", path);
        let mut result = Self {
            level: HashMap::new(),
            subtrees: HashMap::new(),
        };
        let paths = fs::read_dir(path).unwrap();
        for path in paths {
            match std::fs::File::open(path.unwrap().path()) {
                Ok(mut file_handle) => {
                    let m: serde_json::Value = serde_json::from_reader(&file_handle)
                        .expect("Invalidly formatted JSON raw file");
                    let path_string = m
                        .get("path")
                        .expect("Raw JSON file missing path")
                        .as_str()
                        .expect("Raw JSON file path was not a string")
                        .split("/");
                    let mut compiled_path_string: Vec<String> = vec![];
                    for s in path_string {
                        compiled_path_string.push(s.to_owned());
                    }
                    println!("compiled path string {:?}", compiled_path_string);
                    result.insert(
                        Raw::new(compiled_path_string.clone(), m),
                        &compiled_path_string,
                    );
                }
                Err(_) => {}
            }
        }
        result
    }
    fn insert(&mut self, raw: Raw, path_remaining: &[String]) {
        if path_remaining.len() == 1 {
            self.level.insert(path_remaining[0].clone(), raw);
        } else {
            let mut t = self
                .subtrees
                .entry(path_remaining[0].clone())
                .or_insert_with(|| {
                    let mut r = RawTree {
                        level: HashMap::new(),
                        subtrees: HashMap::new(),
                    };
                    r
                });
            t.insert(raw, &path_remaining[1..path_remaining.len()]);
        }
    }
    pub fn search_for_all(&self, path_remaining: &[String]) -> Vec<&Raw> {
        if path_remaining.len() == 0 {
            return self.level.values().collect();
        } else {
            println!("{:?} 3 {:?}", &path_remaining[0], self.subtrees);
            match self.subtrees.get(&path_remaining[0]) {
                Some(tree) => return tree.search_for_all(&path_remaining[1..path_remaining.len()]),
                None => {
                    return self.level.values().collect();
                }
            }
        }
        Vec::new()
    }
    pub fn search(&self, path_remaining: &[String]) -> Option<&Raw> {
        if path_remaining.len() == 1 {
            return self.level.get(&path_remaining[0]);
        } else {
            println!("{:?} 3 {:?}", &path_remaining[0], self.subtrees);
            match self.subtrees.get(&path_remaining[0]) {
                Some(tree) => return tree.search(&path_remaining[1..path_remaining.len()]),
                None => {
                    return None;
                }
            }
        }
        None
    }
}

fn split_path(path: &str) -> Vec<String> {
    let mut v = Vec::new();
    for s in path.split("/") {
        v.push(s.to_owned());
    }
    v
}

#[derive(Debug)]
pub struct Raw {
    dat: Value,
    path: Vec<String>,
}

impl Raw {
    fn new(path: Vec<String>, dat: Value) -> Self {
        Self {
            dat: dat,
            path: path,
        }
    }
    pub fn dat(&self) -> &Value {
        &self.dat
    }
    pub fn get<RawType : 'static + DeserializeOwned> (&self) -> Option<RawType> {
        if let Ok(res) = serde_json::from_value(self.dat.clone()) {
            Some(res)
        } else {
            None
        }
    }
    pub fn path(&self) -> &Vec<String> {
        &self.path
    }

}

#[test]
fn raw_search_all() {
    let rt = RawTree::new("C:\\Users\\justin\\Code\\mmo\\raws");
    println!("len {}", rt.search_for_all(&["one".to_owned()]).len())
}
