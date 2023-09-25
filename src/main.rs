use cowffs::FileSys;

fn main() {
    let mut args = std::env::args(); args.next();
    let Some(fsys_path) = args.next() else {
        return;
    };
    let fsys = FileSys::fs_read_or_create(fsys_path.into()).unwrap();
    fsys.fs_write().unwrap();
}