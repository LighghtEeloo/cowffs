// use std::collections::HashMap;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileSystemError {
    #[error("invalid path segment: `{0}`")]
    InvalidSegment(String),
    #[error("empty path segment")]
    EmptySegment,
    #[error("path starting without `/`")]
    PathStartingWithoutSlash,
    #[error("path ending with `/`")]
    PathEndingWithSlash,
}

/* -------------------------------- interface ------------------------------- */

pub trait IPath:
    TryFrom<Self::Raw, Error = FileSystemError> + IntoIterator<Item = Self::Segment> + Sized
{
    type Raw;
    type Segment;
    type Iter: Iterator<Item = Self::Segment>;

    /* ------------------------------ constructors ------------------------------ */
    fn append(self, raw_segment: impl AsRef<Self::Raw>) -> Result<Self, FileSystemError>;

    /* ------------------------------- destructors ------------------------------ */
    fn parent(self) -> Option<(Self, Self::Segment)>;
    fn iter_parent(&self) -> Self::Iter;
    fn iter(&self) -> Self::Iter;
}

pub trait IMeta {
    // any node should fall into one of the following categories
    fn is_file(&self) -> bool;
    fn is_dir(&self) -> bool;
}

pub trait IFileSystem {
    type Path: IPath;
    type Meta: IMeta;
    type Data;

    /* --------------------------- metadata operations -------------------------- */
    fn metadata(&self, path: Self::Path) -> Result<Self::Meta, FileSystemError>;

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

/* ----------------------------- implementation ----------------------------- */

pub struct FileName(String);

impl FileName {
    pub fn new(raw: impl AsRef<str>) -> Result<Self, FileSystemError> {
        let raw = raw.as_ref();
        if raw.is_empty() {
            Err(FileSystemError::EmptySegment)?
        }
        if raw.contains('/') {
            Err(FileSystemError::InvalidSegment(raw.to_owned()))?
        }
        Ok(Self(raw.to_owned()))
    }
    pub fn to_string(self) -> String {
        self.0
    }
}

pub struct FsPath(Vec<String>, String);

impl TryFrom<String> for FsPath {
    type Error = FileSystemError;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        if !raw.starts_with('/') {
            Err(FileSystemError::PathStartingWithoutSlash)?
        }
        if raw.ends_with('/') {
            Err(FileSystemError::PathEndingWithSlash)?
        }
        let mut path = raw[1..]
            .split('/')
            .map(|s| {
                if s.is_empty() {
                    Err(FileSystemError::EmptySegment)
                } else {
                    Ok(s.to_owned())
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        let Some(last) = path.pop() else {
            unreachable!()
        };
        Ok(Self(path, last))
    }
}

impl IntoIterator for FsPath {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl IPath for FsPath {
    type Raw = String;
    type Segment = String;
    type Iter = std::vec::IntoIter<String>;

    fn append(mut self, raw_segment: impl AsRef<Self::Raw>) -> Result<Self, FileSystemError> {
        let segment = FileName::new(raw_segment.as_ref())?.to_string();
        self.0.push(segment);
        Ok(self)
    }

    fn parent(mut self) -> Option<(Self, Self::Segment)> {
        let last = self.0.pop()?;
        Some((self, last))
    }

    fn iter_parent(&self) -> Self::Iter {
        self.0.clone().into_iter()
    }

    fn iter(&self) -> Self::Iter {
        let mut iter = self.0.clone().into_iter();
        iter.next_back();
        iter
    }
}

pub enum NodeType {
    File,
    Dir,
}

impl IMeta for NodeType {
    fn is_file(&self) -> bool {
        matches!(self, Self::File)
    }

    fn is_dir(&self) -> bool {
        matches!(self, Self::Dir)
    }
}

// pub struct Data(Vec<u8>);

// pub enum Node {
//     File(Data),
//     Dir(HashMap<String, Type>),
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
