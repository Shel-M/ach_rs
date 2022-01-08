use ach_lib_rs::ach_file::AchFile;
use ach_lib_rs::ach_transformations::Transformations;
use std::env;
use std::io;
use std::path::Path;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!(
            "Usage: {} <ach file to process> <config file in ./cfg directory>",
            args[0]
        )
    } else {
        let operations = Transformations::try_from((format!("./cfg/{}", &args[2])).as_ref())?;

        println!("ops: {:?}", operations);

        let ach = AchFile::try_from(Path::new(&args[1])).unwrap();
        print!("{}", ach);

        println!("{}", ach.len());
    }

    Ok(())
}
