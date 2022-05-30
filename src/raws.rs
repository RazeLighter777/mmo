use std::{collections::HashMap, io::Read};

use serde_json::Value;
use std::fs;



#[derive(Debug)]
pub struct RawTree {
    level : HashMap<String,Raw>,
    subtrees : HashMap<String,RawTree>
}

impl RawTree {
    pub fn new(path : &str) -> RawTree {
        println!("The raw path is {}", path);
        let mut result = Self { level : HashMap::new(), subtrees : HashMap::new() };
        let paths = fs::read_dir(path).unwrap();
        for path in paths {
            match std::fs::File::open(path.unwrap().path()) {
                Ok(mut file_handle) => {
                    let m : serde_json::Value =  serde_json::from_reader(&file_handle).expect("Invalidly formatted JSON raw file");
                    let path_string = m.get("path").expect("Raw JSON file missing path").as_str().expect("Raw JSON file path was not a string").split("/");
                    let mut compiled_path_string : Vec<String> = vec![];
                    for s in path_string {
                        compiled_path_string.push(s.to_owned());
                    }
                    println!("compiled path string {:?}", compiled_path_string);
                    result.insert(Raw::new(compiled_path_string.clone(), m), &compiled_path_string);
                }
                Err(_) => {

                },
            }

        }
        result
    }
    fn insert(&mut self, raw : Raw, path_remaining : &[String]) {
        if path_remaining.len() == 1 {
            self.level.insert(path_remaining[0].clone(), raw);
        } else {
            let mut t = RawTree { level: HashMap::new(), subtrees:  HashMap::new() };
            t.insert(raw, &path_remaining[1..path_remaining.len()]);
            self.subtrees.insert(path_remaining[0].clone(), t);
        }
    }
    pub fn search(&self, path_remaining : &[String]) -> Option<&Raw> {
        if path_remaining.len() == 1 {
            return self.level.get(&path_remaining[0])
        } else {
            println!("{:?} 3 {:?}", &path_remaining[0], self.subtrees);
            match self.subtrees.get(&path_remaining[0]) {
                Some(tree) => {
                    return tree.search(&path_remaining[1..path_remaining.len()])
                },
                None => {
                    return None;
                },
            }
        }
        None
    }
}

fn split_path(path : &str) -> Vec<String> {
    let mut v = Vec::new();
    for s in path.split("/"){
        v.push(s.to_owned());
    }
    v
}

#[derive(Debug)]
pub struct Raw {
    dat : Value,
    path : Vec<String>,
}

impl Raw { 
    fn new(path: Vec<String>, dat : Value) -> Self {
        Self { dat : dat, path : path}
    }
    pub fn dat(&self) -> &Value {
        &self.dat
    }
    pub fn path(&self) -> &Vec<String> {
        &self.path
    }
}
