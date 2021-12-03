use ach_lib_rs::ach_file::AchFile;
use std::env;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    let ach = AchFile::try_from(Path::new(&args[1])).unwrap();
    print!("{}", ach);

    println!("{}", ach.len())
}
