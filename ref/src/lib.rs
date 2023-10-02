use std::{collections::HashMap, fmt::Display};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileSystemError {
    #[error("invalid path segment: `{0}`")]
    InvalidSegment(String),
    #[error("empty path segment")]
    EmptySegment,
    #[error("path starting without `/`")]
    PathStartingWithoutSlash,
    #[error("indexing on file: `{0}`")]
    IndexOnFile(String),
    #[error("file `{0}` not found in directory")]
    FileNotInDir(String),
    #[error("cannot operate on root directory")]
    OperateOnRoot,
    #[error("cannot do file operation on a dir node: `{0}`")]
    OperateFileOnDir(String),
    #[error("cannot do dir operation on a file node: `{0}`")]
    OperateDirOnFile(String),
}

/* -------------------------------- interface ------------------------------- */

pub trait IPath:
    TryFrom<Self::Raw, Error = FileSystemError> + IntoIterator<Item = Self::Segment> + Sized
{
    type Raw;
    type Segment;
    // type Iter: Iterator<Item = Self::Segment>;

    /* ------------------------------ constructors ------------------------------ */
    fn append(self, raw_segment: &Self::Raw) -> Result<Self, FileSystemError>;

    /* ------------------------------- destructors ------------------------------ */
    fn parent(self) -> Option<(Self, Self::Segment)>;
    // fn iter(&self) -> Self::Iter;
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

    fn init() -> Self;

    /* --------------------------- metadata operations -------------------------- */
    fn metadata(&self, path: Self::Path) -> Result<Self::Meta, FileSystemError>;

    /* ----------------------------- file operations ---------------------------- */
    fn create_file(&mut self, path: Self::Path) -> Result<(), FileSystemError>;
    fn read_file(&self, path: Self::Path) -> Result<Self::Data, FileSystemError>;
    fn write_file(&mut self, path: Self::Path, data: Self::Data) -> Result<(), FileSystemError>;
    fn remove_file(&mut self, path: Self::Path) -> Result<(), FileSystemError>;

    /* -------------------------- directory operations -------------------------- */
    fn create_dir(&mut self, path: Self::Path) -> Result<(), FileSystemError>;
    fn read_dir(&self, path: Self::Path) -> Result<Vec<<Self::Path as IPath>::Segment>, FileSystemError>;
    fn remove_dir(&mut self, path: Self::Path) -> Result<(), FileSystemError>;

    /* ----------------------------- link operations ---------------------------- */
    fn create_link(&mut self, path: Self::Path, target: Self::Path) -> Result<(), FileSystemError>;
}

/* ----------------------------- implementation ----------------------------- */

#[derive(Clone)]
pub struct FileName(String);

impl Display for FileName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

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
}

#[derive(Clone)]
pub struct FsPath(Vec<FileName>);

impl Display for FsPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut path = String::new();
        for segment in &self.0 {
            path.push('/');
            path.push_str(&segment.to_string());
        }
        write!(f, "{}", path)
    }
}

impl TryFrom<String> for FsPath {
    type Error = FileSystemError;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        if !raw.starts_with('/') {
            Err(FileSystemError::PathStartingWithoutSlash)?
        }
        let path = raw[1..]
            .split('/')
            .map(FileName::new)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self(path))
    }
}

impl IntoIterator for FsPath {
    type Item = FileName;
    type IntoIter = std::vec::IntoIter<FileName>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl IPath for FsPath {
    type Raw = String;
    type Segment = FileName;
    // type Iter = std::vec::IntoIter<String>;

    fn append(mut self, raw_segment: &Self::Raw) -> Result<Self, FileSystemError> {
        let segment = FileName::new(raw_segment)?;
        self.0.push(segment);
        Ok(self)
    }

    fn parent(mut self) -> Option<(Self, Self::Segment)> {
        let last = self.0.pop()?;
        Some((self, last))
    }

    // fn iter(&self) -> Self::Iter {
    //     self.0.clone().into_iter()
    // }
}

#[derive(Clone)]
pub struct Data(Vec<u8>);

#[derive(Clone)]
pub enum Node {
    File(Data),
    Dir(HashMap<String, Node>),
}

impl Node {
    pub fn child_node(&self, name: FileName) -> Result<&Self, FileSystemError> {
        match self {
            Self::File(_) => Err(FileSystemError::IndexOnFile(name.to_string())),
            Self::Dir(children) => children
                .get(&name.to_string())
                .ok_or(FileSystemError::FileNotInDir(name.to_string())),
        }
    }
}

impl IMeta for Node {
    fn is_file(&self) -> bool {
        matches!(self, Self::File(_))
    }

    fn is_dir(&self) -> bool {
        matches!(self, Self::Dir(_))
    }
}

pub struct ReffFs {
    pub root: HashMap<String, Node>,
}

impl ReffFs {
    pub fn traverse(&self, path: FsPath) -> Result<Node, FileSystemError> {
        let mut dir = Node::Dir(self.root.clone());
        for segment in path {
            dir = dir.child_node(segment)?.clone();
        }
        Ok(dir.clone())
    }
    pub fn traverse_mut(
        &mut self, path: FsPath,
    ) -> Result<&mut HashMap<String, Node>, FileSystemError> {
        let mut dir = &mut self.root;
        for segment in path {
            dir = match dir
                .get_mut(&segment.to_string())
                .ok_or(FileSystemError::FileNotInDir(segment.to_string()))?
            {
                Node::File(_) => Err(FileSystemError::IndexOnFile(segment.to_string()))?,
                Node::Dir(children) => children,
            }
        }
        Ok(dir)
    }
}

impl IFileSystem for ReffFs {
    type Path = FsPath;

    type Meta = Node;

    type Data = Data;

    fn init() -> Self {
        let root = HashMap::new();
        Self { root }
    }

    fn metadata(&self, path: Self::Path) -> Result<Self::Meta, FileSystemError> {
        self.traverse(path)
    }

    fn create_file(&mut self, path: Self::Path) -> Result<(), FileSystemError> {
        let (parent, name) = path.parent().ok_or(FileSystemError::OperateOnRoot)?;
        let dir = self.traverse_mut(parent)?;
        dir.insert(name.to_string(), Node::File(Data(vec![])));
        Ok(())
    }

    fn read_file(&self, path: Self::Path) -> Result<Self::Data, FileSystemError> {
        let node = self.traverse(path.clone())?;
        match node {
            Node::File(data) => Ok(data),
            Node::Dir(_) => Err(FileSystemError::OperateFileOnDir(path.to_string())),
        }
    }

    fn write_file(&mut self, path: Self::Path, data: Self::Data) -> Result<(), FileSystemError> {
        let (parent, name) = path
            .clone()
            .parent()
            .ok_or(FileSystemError::OperateOnRoot)?;
        let dir = self.traverse_mut(parent)?;
        let node = dir
            .get_mut(&name.to_string())
            .ok_or(FileSystemError::FileNotInDir(name.to_string()))?;
        match node {
            Node::File(fdata) => *fdata = data.clone(),
            Node::Dir(_) => Err(FileSystemError::OperateFileOnDir(path.to_string()))?,
        }
        Ok(())
    }

    fn remove_file(&mut self, path: Self::Path) -> Result<(), FileSystemError> {
        let (parent, name) = path
            .clone()
            .parent()
            .ok_or(FileSystemError::OperateOnRoot)?;
        let dir = self.traverse_mut(parent)?;
        let node = dir
            .get_mut(&name.to_string())
            .ok_or(FileSystemError::FileNotInDir(name.to_string()))?;
        match node {
            Node::File(_) => {
                dir.remove(&name.to_string());
            }
            Node::Dir(_) => Err(FileSystemError::OperateFileOnDir(path.to_string()))?,
        }
        Ok(())
    }

    fn create_dir(&mut self, path: Self::Path) -> Result<(), FileSystemError> {
        let (parent, name) = path.parent().ok_or(FileSystemError::OperateOnRoot)?;
        let dir = self.traverse_mut(parent)?;
        dir.insert(name.to_string(), Node::Dir(HashMap::new()));
        Ok(())
    }

    fn read_dir(&self, path: Self::Path) -> Result<Vec<FileName>, FileSystemError> {
        let node = self.traverse(path.clone())?;
        match node {
            Node::File(_) => Err(FileSystemError::OperateFileOnDir(path.to_string())),
            Node::Dir(children) => children
                .into_iter()
                .map(|(name, _)| FileName::new(name))
                .collect(),
        }
    }

    fn remove_dir(&mut self, path: Self::Path) -> Result<(), FileSystemError> {
        let (parent, name) = path
            .clone()
            .parent()
            .ok_or(FileSystemError::OperateOnRoot)?;
        let dir = self.traverse_mut(parent)?;
        let node = dir
            .get_mut(&name.to_string())
            .ok_or(FileSystemError::FileNotInDir(name.to_string()))?;
        match node {
            Node::File(_) => Err(FileSystemError::OperateFileOnDir(path.to_string()))?,
            Node::Dir(children) => {
                if !children.is_empty() {
                    Err(FileSystemError::OperateDirOnFile(path.to_string()))?
                }
                dir.remove(&name.to_string());
            }
        }
        Ok(())
    }

    fn create_link(&mut self, _path: Self::Path, _target: Self::Path) -> Result<(), FileSystemError> {
        todo!()
    }
}
