use cowffs::FileSys;

fn main() {
    let mut args = std::env::args();
    args.next();
    let Some(fsys_path) = args.next() else {
        return;
    };
    let fsys = FileSys::fs_disk_load(fsys_path.into()).unwrap();
    fsys.fs_disk_dump().unwrap();
}
