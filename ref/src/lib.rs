// use std::collections::HashMap;

// pub struct Data(Vec<u8>);

// pub enum Node {
//     File(Data),
//     Dir(HashMap<String, Type>),
// }

// pub enum Type {
//     File,
//     Dir,
// }

// pub struct FileSys {
//     pub map: HashMap<String, Node>,
// }

// impl FileSys {
//     pub fn new() -> Self {
//         let mut map = HashMap::new();
//         map.insert("/".to_owned(), Node::Dir(HashMap::new()));
//         FileSys { map }
//     }
// }

use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileSystemError {
    #[error("invalid path segment: {0}")]
    InvalidSegment(String),
}

pub trait IPath:
    TryFrom<Self::Raw, Error = FileSystemError> + IntoIterator<Item = Self::Segment> + Sized
{
    type Raw;
    type Segment;
    type Iter: Iterator<Item = Self::Segment>;

    /* ------------------------------ constructors ------------------------------ */
    fn root() -> Self;
    fn append(self, raw_segment: impl AsRef<Self::Raw>) -> Result<Self, FileSystemError>;

    /* ------------------------------- destructors ------------------------------ */
    fn parent(self) -> Option<(Self, Self::Segment)>;
    fn iter_parent(&self) -> Self::Iter;
    fn iter(&self) -> Self::Iter;
}

pub trait IFileSystem {
    type Path: IPath;
    type Data;

    /* ----------------------------- file operations ---------------------------- */
    fn create_file(&mut self, path: Self::Path) -> Result<(), FileSystemError>;
    fn read_file(&self, path: Self::Path) -> Result<Self::Data, FileSystemError>;
    fn write_file(&mut self, path: Self::Path, data: Self::Data) -> Result<(), FileSystemError>;
    fn remove_file(&mut self, path: Self::Path) -> Result<(), FileSystemError>;

    /* -------------------------- directory operations -------------------------- */
    fn create_dir(&mut self, path: Self::Path) -> Result<(), FileSystemError>;
    fn read_dir(&self, path: Self::Path) -> Result<Vec<Self::Path>, FileSystemError>;
    fn remove_dir(&mut self, path: Self::Path) -> Result<(), FileSystemError>;

    /* ----------------------------- link operations ---------------------------- */
    fn create_link(&mut self, path: Self::Path, target: Self::Path) -> Result<(), FileSystemError>;
}
