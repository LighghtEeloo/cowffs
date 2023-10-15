use refffs::*;

fn main() -> Result<(), FileSystemError<FsPath>> {
    let mut fs = ReffFs::init();
    fs.create_file(FsPath::try_from("/a.txt")?)?;
    fs.create_file(FsPath::try_from("/b.txt")?)?;
    fs.create_dir(FsPath::try_from("/c")?)?;
    fs.create_file(FsPath::try_from("/c/d.txt")?)?;
    fs.create_file(FsPath::try_from("/c/e.txt")?)?;
    println!("=== /c ===");
    let items = fs.read_dir(FsPath::try_from("/c")?)?;
    for item in items {
        println!("{}", item);
    }
    println!("=== / ===");
    let items = fs.read_dir(FsPath::try_from("/")?)?;
    for item in items {
        println!("{}", item);
    }
    Ok(())
}
