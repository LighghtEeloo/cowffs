use refffs::*;

fn main() -> Result<(), FileSystemError<FsPath>> {
    let mut fs = FileSystem::init();
    fs.create_file(FsPath::try_from("/a.txt")?)?;
    fs.create_file(FsPath::try_from("/b.txt")?)?;
    fs.create_dir(FsPath::try_from("/c")?)?;
    fs.create_file(FsPath::try_from("/c/d.txt")?)?;
    fs.create_file(FsPath::try_from("/c/e.txt")?)?;
    println!("=== [read] /c ===");
    let items = fs.read_dir(FsPath::try_from("/c")?)?;
    for item in items {
        println!("{}", item);
    }
    println!("=== [read] / ===");
    let items = fs.read_dir(FsPath::try_from("/")?)?;
    for item in items {
        println!("{}", item);
    }
    fs.read_dir(FsPath::try_from("/not_exist")?).expect_err("should fail");
    fs.read_dir(FsPath::try_from("/c/not_exist")?).expect_err("should fail");
    let data = fs.read_file(FsPath::try_from("/a.txt")?)?;
    println!("=== [read] /a.txt ===");
    println!("{}", data);
    fs.write_file(FsPath::try_from("/a.txt")?, Data::new("Hello, world!".as_bytes()))?;
    let data = fs.read_file(FsPath::try_from("/a.txt")?)?;
    println!("=== [read] /a.txt ===");
    println!("{}", data);
    fs.remove(FsPath::try_from("/a.txt")?)?;
    fs.remove(FsPath::try_from("/c/not_exist")?).expect_err("should fail");
    fs.remove(FsPath::try_from("/c")?).expect_err("should fail");
    fs.remove(FsPath::try_from("/not_exist")?).expect_err("should fail");
    Ok(())
}
