use std::fs::File;
use std::io;
use std::io::{BufReader, BufRead};

#[derive(Debug)]
pub enum OperationType {
	SPLIT
}

#[derive(Debug)]
struct KeyVal {
	key: String,
	val: String,
}

#[derive(Debug)]
pub struct Steps {
	pub label: String,
	pub operation: (Operation, String),
	pub conditions: Vec<KeyVal>,
	pub replacements: Vec<KeyVal>,
}


//todo: work out indentation levels for step parts. Basically: parse the yaml config.
// This might end up tough to implement, but the flexibility is necessary.

pub fn read_cfg(file: &String) -> io::Result<Vec<Steps>> {
	let file = File::open(format!("./cfg/{}", file))?;
	let lines = BufReader::new(file).lines();
	
	let mut steps = vec![];
	for line in lines {
		let mut comment_char_found = false;
		let line: String = line?.chars().filter(|c| {
			return if *c == '#' || comment_char_found {
				comment_char_found = true;
				false
			} else {
				true
			};
		}).collect();
		if line.len() == 0 {
			continue
		}
		
		
	}
	
	Ok(steps)
}
