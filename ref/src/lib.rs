mod view;

use std::collections::HashMap;

use view::FPath;

pub struct Data(Vec<u8>);

pub enum Node {
    File(Data),
    Dir(HashMap<String, Type>),
}

pub enum Type {
    File,
    Dir,
}

pub struct FileSys {
    pub map: HashMap<String, Node>,
}

impl FileSys {
    pub fn new() -> Self {
        let mut map = HashMap::new();
        map.insert("/".to_owned(), Node::Dir(HashMap::new()));
        FileSys { map }
    }

    /* ----------------------------- file operations ---------------------------- */
    pub fn create_file(&mut self, path: String) -> anyhow::Result<()>{
        todo!()
    }
    pub fn read_file(&self, path: String) -> anyhow::Result<Data> {
        todo!()
    }
    pub fn write_file(&mut self, path: String, data: Data) -> anyhow::Result<()> {
        todo!()
    }
    pub fn remove_file(&mut self, path: String) -> anyhow::Result<()> {
        todo!()
    }

    /* -------------------------- directory operations -------------------------- */
    pub fn create_dir(&mut self, path: String) -> anyhow::Result<()>{
        todo!()
    }
    pub fn read_dir(&self, path: String) -> anyhow::Result<Vec<String>> {
        todo!()
    }
    pub fn remove_dir(&mut self, path: String) -> anyhow::Result<()> {
        todo!()
    }

    /* ----------------------------- link operations ---------------------------- */
    pub fn create_link(&mut self, path: String, target: String) -> anyhow::Result<()>{
        todo!()
    }
}
