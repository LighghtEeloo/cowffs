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
    IndexOnFile(FsPath),
    #[error("file `{0}` not found in directory")]
    FileNotInDir(FsPath),
    #[error("cannot operate on root directory")]
    OperateOnRoot,
    #[error("cannot do file operation on a dir node: `{0}`")]
    OperateFileOnDir(FsPath),
    #[error("cannot do dir operation on a file node: `{0}`")]
    OperateDirOnFile(FsPath),
    #[error("cannot remove non-empty directory: `{0}`")]
    RemoveNonEmptyDir(FsPath),
}

/* -------------------------------- interface ------------------------------- */

pub trait IPath:
    Sized + TryFrom<Self::Raw, Error = FileSystemError> + IntoIterator<Item = Self::Segment> + Display
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

    /* -------------------------- directory operations -------------------------- */
    fn create_dir(&mut self, path: Self::Path) -> Result<(), FileSystemError>;
    fn read_dir(
        &self, path: Self::Path,
    ) -> Result<Vec<<Self::Path as IPath>::Segment>, FileSystemError>;

    /* ----------------------------- remove operations --------------------------- */
    fn remove(&mut self, path: Self::Path) -> Result<(), FileSystemError>;

    /* ----------------------------- link operations ---------------------------- */
    fn create_link(&mut self, path: Self::Path, target: Self::Path) -> Result<(), FileSystemError>;
}

/* ----------------------------- implementation ----------------------------- */

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
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

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NodeId(usize);

#[derive(Clone)]
pub struct Node {
    // cnt: usize,
    inner: NodeInner,
}
#[derive(Clone)]
pub enum NodeInner {
    File(Data),
    Dir(HashMap<String, NodeId>),
}

impl Node {
    pub fn with_inner(inner: NodeInner) -> Self {
        Self {
            // cnt: 1,
            inner,
        }
    }
    pub fn dir(&self, path: FsPath) -> Result<&HashMap<String, NodeId>, FileSystemError> {
        match &self.inner {
            NodeInner::File(_) => Err(FileSystemError::IndexOnFile(path)),
            NodeInner::Dir(children) => Ok(children),
        }
    }
    pub fn dir_mut(
        &mut self, path: FsPath,
    ) -> Result<&mut HashMap<String, NodeId>, FileSystemError> {
        match &mut self.inner {
            NodeInner::File(_) => Err(FileSystemError::IndexOnFile(path)),
            NodeInner::Dir(children) => Ok(children),
        }
    }
    pub fn file(&self, path: FsPath) -> Result<&Data, FileSystemError> {
        match &self.inner {
            NodeInner::File(data) => Ok(data),
            NodeInner::Dir(_) => Err(FileSystemError::OperateDirOnFile(path)),
        }
    }
    pub fn file_mut(&mut self, path: FsPath) -> Result<&mut Data, FileSystemError> {
        match &mut self.inner {
            NodeInner::File(data) => Ok(data),
            NodeInner::Dir(_) => Err(FileSystemError::OperateDirOnFile(path)),
        }
    }
}

impl IMeta for Node {
    fn is_file(&self) -> bool {
        matches!(self.inner, NodeInner::File(_))
    }

    fn is_dir(&self) -> bool {
        matches!(self.inner, NodeInner::Dir(_))
    }
}

pub struct ReffFs {
    pub nodes: Vec<Node>,
    pub root: NodeId,
}

impl std::ops::Index<NodeId> for ReffFs {
    type Output = Node;

    fn index(&self, index: NodeId) -> &Self::Output {
        &self.nodes[index.0]
    }
}

impl std::ops::IndexMut<NodeId> for ReffFs {
    fn index_mut(&mut self, index: NodeId) -> &mut Self::Output {
        &mut self.nodes[index.0]
    }
}

impl ReffFs {
    pub fn traverse(&self, path: FsPath) -> Result<&Node, FileSystemError> {
        let node = self.traverse_id(path)?;
        Ok(&self[node])
    }
    pub fn traverse_dir_mut(
        &mut self, path: FsPath,
    ) -> Result<&mut HashMap<String, NodeId>, FileSystemError> {
        let dir = self.traverse_id(path.clone())?;
        Ok(self[dir].dir_mut(path.clone())?)
    }
    pub fn traverse_id(&self, path: FsPath) -> Result<NodeId, FileSystemError> {
        let mut dir = self.root;
        for segment in path.clone() {
            let next = *self[dir]
                .dir(path.clone())?
                .get(&segment.to_string())
                .ok_or(FileSystemError::FileNotInDir(path.clone()))?;
            dir = next;
        }
        Ok(dir)
    }
    pub fn fresh(&mut self, node: Node) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(node);
        id
    }
}

impl IFileSystem for ReffFs {
    type Path = FsPath;

    type Meta = Node;

    type Data = Data;

    fn init() -> Self {
        let nodes = vec![Node::with_inner(NodeInner::Dir(HashMap::new()))];
        let root = NodeId(0);
        Self { nodes, root }
    }

    fn metadata(&self, path: Self::Path) -> Result<Self::Meta, FileSystemError> {
        self.traverse(path).cloned()
    }

    fn create_file(&mut self, path: Self::Path) -> Result<(), FileSystemError> {
        let (parent, name) = path.parent().ok_or(FileSystemError::OperateOnRoot)?;
        let new_file = self.fresh(Node::with_inner(NodeInner::File(Data(vec![]))));
        let dir = self.traverse_dir_mut(parent)?;
        dir.insert(name.to_string(), new_file);
        Ok(())
    }

    fn read_file(&self, path: Self::Path) -> Result<Self::Data, FileSystemError> {
        let node = self.traverse(path.clone())?;
        node.file(path.clone()).cloned()
    }

    fn write_file(&mut self, path: Self::Path, data: Self::Data) -> Result<(), FileSystemError> {
        let (parent, name) = path
            .clone()
            .parent()
            .ok_or(FileSystemError::OperateOnRoot)?;
        let dir = self.traverse_dir_mut(parent)?;
        let node = *dir
            .get_mut(&name.to_string())
            .ok_or(FileSystemError::FileNotInDir(path.clone()))?;
        let fdata = self[node].file_mut(path.clone())?;
        *fdata = data.clone();
        Ok(())
    }

    fn create_dir(&mut self, path: Self::Path) -> Result<(), FileSystemError> {
        let (parent, name) = path.parent().ok_or(FileSystemError::OperateOnRoot)?;
        let new_dir = self.fresh(Node::with_inner(NodeInner::Dir(HashMap::new())));
        let dir = self.traverse_dir_mut(parent)?;
        dir.insert(name.to_string(), new_dir);
        Ok(())
    }

    fn read_dir(&self, path: Self::Path) -> Result<Vec<FileName>, FileSystemError> {
        let node = self.traverse(path.clone())?;
        let children = node.dir(path.clone())?;
        children
            .into_iter()
            .map(|(name, _)| FileName::new(name))
            .collect()
    }

    fn remove(&mut self, path: Self::Path) -> Result<(), FileSystemError> {
        let (parent, name) = path
            .clone()
            .parent()
            .ok_or(FileSystemError::OperateOnRoot)?;
        let dir = self.traverse_id(parent)?;
        let node = *self[dir]
            .dir(path.clone())?
            .get(&name.to_string())
            .ok_or(FileSystemError::FileNotInDir(path.clone()))?;
        if !self[node].dir(path.clone())?.is_empty() {
            Err(FileSystemError::RemoveNonEmptyDir(path.clone()))?
        }
        self[dir].dir_mut(path.clone())?.remove(&name.to_string());
        Ok(())
    }

    fn create_link(&mut self, path: Self::Path, target: Self::Path) -> Result<(), FileSystemError> {
        let target = self.traverse_id(target)?;
        let (parent, name) = path
            .clone()
            .parent()
            .ok_or(FileSystemError::OperateOnRoot)?;
        let parent = self.traverse_dir_mut(parent)?;
        parent.insert(name.to_string(), target);
        Ok(())
    }
}
