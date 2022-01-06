mod config;

use ach_lib_rs::ach_file::AchFile;
use config::{Operation, read_cfg};
use std::env;
use std::path::Path;
use std::io;

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
