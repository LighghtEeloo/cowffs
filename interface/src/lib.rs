use std::fmt::Display;
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
    Sized + TryFrom<Self::Raw, Error = FileSystemError<Self>> + IntoIterator<Item = Self::Segment> + Display
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
    fn metadata(&self, path: Self::Path<'fs>) -> Result<Self::Meta, FileSystemError<Self::Path<'fs>>>;

    /* ----------------------------- file operations ---------------------------- */
    fn create_file(&mut self, path: Self::Path<'fs>) -> Result<(), FileSystemError<Self::Path<'fs>>>;
    fn read_file(&self, path: Self::Path<'fs>) -> Result<Self::Data, FileSystemError<Self::Path<'fs>>>;
    fn write_file(&mut self, path: Self::Path<'fs>, data: Self::Data) -> Result<(), FileSystemError<Self::Path<'fs>>>;

    /* -------------------------- directory operations -------------------------- */
    fn create_dir(&mut self, path: Self::Path<'fs>) -> Result<(), FileSystemError<Self::Path<'fs>>>;
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
