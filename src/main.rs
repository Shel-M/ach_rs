use ach_lib_rs::ach_file::AchFile;
use std::env;
use std::fs::File;
use std::path::Path;
use std::io;
use std::io::BufRead;

#[derive(Debug)]
enum Operation {
    CSplit
}

#[derive(Debug)]
struct Steps {
    label: String,
    op: Operation,
    arg: Vec<String>,
}

fn main() -> io::Result<()>{
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 3 {
        println!("Usage: {} <ach file to process> <config file in ./cfg directory>", args[0])
    }
    else {
        let operations = read_cfg(&args[2])?;
        
        println!("ops: {:?}", operations);
        
        let ach = AchFile::try_from(Path::new(&args[1])).unwrap();
        print!("{}", ach);
    
        println!("{}", ach.len());
        
        for operation in operations {
            print!("performing operation {} ", operation.label);
            match operation.op {
                Operation::CSplit => {
                    println!("company split");
                    let _ = ach.split(operation.arg);
                }
            }
        }
    }
    
    Ok(())
}

fn read_cfg(file: &String) -> io::Result<Vec<Steps>> {
    let file = File::open(format!("./cfg/{}", file))?;
    let lines = io::BufReader::new(file).lines();
    
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
        
        let words: Vec<String> = line.split(" ").map(|s| s.to_string()).filter(|s| s != "").collect();
    
        match &*words[1] {
            "include" => {
                steps.append(&mut read_cfg(&words[1])?);
            }
            s => {
                match s {
                    "csplit" => {
                        steps.push(
                            Steps {
                                label: words[0].clone().to_string(),
                                op: Operation::CSplit,
                                arg: words[2..words.len()].to_vec(),
                            }
                        )
                    }
                    _ => {}
                }
            }
        }
        
    }
    
    Ok(steps)
}
