use std::{collections::HashMap, fmt::Display};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileSystemError<P: Display> {
    #[error("invalid path segment: `{0}`")]
    InvalidSegment(String),
    #[error("empty path segment")]
    EmptySegment,
    #[error("path starting without `/`")]
    PathStartingWithoutSlash,
    #[error("indexing on file: `{0}`")]
    IndexOnFile(P),
    #[error("file `{0}` not found in directory")]
    FileNotInDir(P),
    #[error("cannot operate on root directory")]
    OperateOnRoot,
    #[error("cannot do file operation on a dir node: `{0}`")]
    OperateFileOnDir(P),
    #[error("cannot do dir operation on a file node: `{0}`")]
    OperateDirOnFile(P),
    #[error("cannot remove non-empty directory: `{0}`")]
    RemoveNonEmptyDir(P),
}

/* -------------------------------- interface ------------------------------- */

pub trait IPath<'p>:
    Sized
    + TryFrom<Self::Raw, Error = FileSystemError<Self>>
    + IntoIterator<Item = Self::Segment>
    + Display
{
    type Raw;
    type Segment: 'p;
    type Iter: Iterator<Item = &'p Self::Segment>;

    /* ------------------------------ constructors ------------------------------ */
    fn append(self, raw_segment: &Self::Raw) -> Result<Self, FileSystemError<Self>>;

    /* ------------------------------- destructors ------------------------------ */
    fn parent(self) -> Option<(Self, Self::Segment)>;
    fn iter(&'p self) -> Self::Iter;
}

pub trait IMeta {
    // any node should fall into one of the following categories
    fn is_file(&self) -> bool;
    fn is_dir(&self) -> bool;
}

pub trait IFileSystem<'fs> {
    type Path<'p>: IPath<'p>;
    type Meta: IMeta;
    type Data;

    fn init() -> Self;

    /* --------------------------- metadata operations -------------------------- */
    fn metadata(
        &self, path: Self::Path<'fs>,
    ) -> Result<Self::Meta, FileSystemError<Self::Path<'fs>>>;

    /* ----------------------------- file operations ---------------------------- */
    fn create_file(
        &mut self, path: Self::Path<'fs>,
    ) -> Result<(), FileSystemError<Self::Path<'fs>>>;
    fn read_file(
        &self, path: Self::Path<'fs>,
    ) -> Result<Self::Data, FileSystemError<Self::Path<'fs>>>;
    fn write_file(
        &mut self, path: Self::Path<'fs>, data: Self::Data,
    ) -> Result<(), FileSystemError<Self::Path<'fs>>>;

    /* -------------------------- directory operations -------------------------- */
    fn create_dir(&mut self, path: Self::Path<'fs>)
        -> Result<(), FileSystemError<Self::Path<'fs>>>;
    fn read_dir(
        &self, path: Self::Path<'fs>,
    ) -> Result<Vec<<Self::Path<'fs> as IPath<'fs>>::Segment>, FileSystemError<Self::Path<'fs>>>;

    /* ----------------------------- link operations ---------------------------- */
    fn create_link(
        &mut self, path: Self::Path<'fs>, target: Self::Path<'fs>,
    ) -> Result<(), FileSystemError<Self::Path<'fs>>>;

    /* ----------------------------- remove operations --------------------------- */
    fn remove(&mut self, path: Self::Path<'fs>) -> Result<(), FileSystemError<Self::Path<'fs>>>;
}

/* ----------------------------- implementation ----------------------------- */

type ReffFsError = FileSystemError<FsPath>;

#[derive(Clone, Debug)]
pub struct FileName(String);

impl Display for FileName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FileName {
    pub fn new(raw: impl AsRef<str>) -> Result<Self, ReffFsError> {
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
            path += "/";
            path += &segment.to_string();
        }
        if path.is_empty() {
            path += "/";
        }
        write!(f, "{}", path)
    }
}

impl TryFrom<&str> for FsPath {
    type Error = FileSystemError<Self>;

    fn try_from(raw: &str) -> Result<Self, Self::Error> {
        if !raw.starts_with('/') {
            Err(FileSystemError::PathStartingWithoutSlash)?
        }
        if raw == "/" {
            return Ok(Self(vec![]));
        }
        let path = raw[1..]
            .split('/')
            .map(FileName::new)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self(path))
    }
}

impl TryFrom<String> for FsPath {
    type Error = FileSystemError<Self>;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        Self::try_from(raw.as_str())
    }
}

impl IntoIterator for FsPath {
    type Item = FileName;
    type IntoIter = std::vec::IntoIter<FileName>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'p> IPath<'p> for FsPath {
    type Raw = String;
    type Segment = FileName;
    type Iter = std::slice::Iter<'p, FileName>;

    fn append(mut self, raw_segment: &Self::Raw) -> Result<Self, ReffFsError> {
        let segment = FileName::new(raw_segment)?;
        self.0.push(segment);
        Ok(self)
    }

    fn parent(mut self) -> Option<(Self, Self::Segment)> {
        let last = self.0.pop()?;
        Some((self, last))
    }

    fn iter(&'p self) -> Self::Iter {
        self.0.iter()
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Data(Vec<u8>);

impl Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut data = String::new();
        for byte in &self.0 {
            data += &format!("{:02x}", byte);
        }
        write!(f, "{}", data)
    }
}

impl Data {
    pub fn new(raw: impl AsRef<[u8]>) -> Self {
        Self(raw.as_ref().to_vec())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NodeId(usize);

#[derive(Clone, Debug)]
pub struct Node {
    // cnt: usize,
    inner: NodeInner,
}
#[derive(Clone, Debug)]
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
    pub fn dir(&self, path: FsPath) -> Result<&HashMap<String, NodeId>, ReffFsError> {
        match &self.inner {
            NodeInner::File(_) => Err(FileSystemError::IndexOnFile(path)),
            NodeInner::Dir(children) => Ok(children),
        }
    }
    pub fn dir_mut(&mut self, path: FsPath) -> Result<&mut HashMap<String, NodeId>, ReffFsError> {
        match &mut self.inner {
            NodeInner::File(_) => Err(FileSystemError::IndexOnFile(path)),
            NodeInner::Dir(children) => Ok(children),
        }
    }
    pub fn file(&self, path: FsPath) -> Result<&Data, ReffFsError> {
        match &self.inner {
            NodeInner::File(data) => Ok(data),
            NodeInner::Dir(_) => Err(FileSystemError::OperateDirOnFile(path)),
        }
    }
    pub fn file_mut(&mut self, path: FsPath) -> Result<&mut Data, ReffFsError> {
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
    pub fn traverse(&self, path: FsPath) -> Result<&Node, ReffFsError> {
        let node = self.traverse_id(path)?;
        Ok(&self[node])
    }
    pub fn traverse_dir_mut(
        &mut self, path: FsPath,
    ) -> Result<&mut HashMap<String, NodeId>, ReffFsError> {
        let dir = self.traverse_id(path.clone())?;
        Ok(self[dir].dir_mut(path.clone())?)
    }
    pub fn traverse_id(&self, path: FsPath) -> Result<NodeId, ReffFsError> {
        let mut current = self.root;
        for segment in path.clone() {
            let next = *self[current]
                .dir(path.clone())?
                .get(&segment.to_string())
                .ok_or(FileSystemError::FileNotInDir(path.clone()))?;
            current = next;
        }
        Ok(current)
    }
    pub fn fresh(&mut self, node: Node) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(node);
        id
    }
}

impl<'fs> IFileSystem<'fs> for ReffFs {
    type Path<'p> = FsPath;

    type Meta = Node;

    type Data = Data;

    fn init() -> Self {
        let nodes = vec![Node::with_inner(NodeInner::Dir(HashMap::new()))];
        let root = NodeId(0);
        Self { nodes, root }
    }

    fn metadata(&self, path: Self::Path<'fs>) -> Result<Self::Meta, ReffFsError> {
        self.traverse(path).cloned()
    }

    fn create_file(&mut self, path: Self::Path<'fs>) -> Result<(), ReffFsError> {
        let (parent, name) = path.parent().ok_or(FileSystemError::OperateOnRoot)?;
        let new_file = self.fresh(Node::with_inner(NodeInner::File(Data(vec![]))));
        let dir = self.traverse_dir_mut(parent)?;
        dir.insert(name.to_string(), new_file);
        Ok(())
    }

    fn read_file(&self, path: Self::Path<'fs>) -> Result<Self::Data, ReffFsError> {
        let node = self.traverse(path.clone())?;
        node.file(path.clone()).cloned()
    }

    fn write_file(&mut self, path: Self::Path<'fs>, data: Self::Data) -> Result<(), ReffFsError> {
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

    fn create_dir(&mut self, path: Self::Path<'fs>) -> Result<(), ReffFsError> {
        let (parent, name) = path.parent().ok_or(FileSystemError::OperateOnRoot)?;
        let new_dir = self.fresh(Node::with_inner(NodeInner::Dir(HashMap::new())));
        let dir = self.traverse_dir_mut(parent)?;
        dir.insert(name.to_string(), new_dir);
        Ok(())
    }

    fn read_dir(&self, path: Self::Path<'fs>) -> Result<Vec<FileName>, ReffFsError> {
        let node = self.traverse(path.clone())?;
        let children = node.dir(path.clone())?;
        children
            .into_iter()
            .map(|(name, _)| FileName::new(name))
            .collect()
    }

    fn create_link(
        &mut self, path: Self::Path<'fs>, target: Self::Path<'fs>,
    ) -> Result<(), ReffFsError> {
        let target = self.traverse_id(target)?;
        let (parent, name) = path
            .clone()
            .parent()
            .ok_or(FileSystemError::OperateOnRoot)?;
        let parent = self.traverse_dir_mut(parent)?;
        parent.insert(name.to_string(), target);
        Ok(())
    }

    fn remove(&mut self, path: Self::Path<'fs>) -> Result<(), ReffFsError> {
        let (parent, name) = path
            .clone()
            .parent()
            .ok_or(FileSystemError::OperateOnRoot)?;
        let dir = self.traverse_id(parent)?;
        let node = *self[dir]
            .dir(path.clone())?
            .get(&name.to_string())
            .ok_or(FileSystemError::FileNotInDir(path.clone()))?;
        if let Ok(dir) = self[node].dir(path.clone()) {
            if !dir.is_empty() {
                Err(FileSystemError::RemoveNonEmptyDir(path.clone()))?
            }
        }
        self[dir].dir_mut(path.clone())?.remove(&name.to_string());
        Ok(())
    }
}
