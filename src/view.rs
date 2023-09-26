pub struct FPath<'a>(pub Vec<&'a str>);
impl<'a> FPath<'a> {
    pub fn new(s: &'a str) -> anyhow::Result<Self> {
        if s.starts_with('/') {
            let x = s[1..]
                .split('/')
                .map(|s| {
                    if s.is_empty() {
                        Err(anyhow::anyhow!("Path must not contain empty components"))
                    } else {
                        Ok(s)
                    }
                })
                .collect::<Result<_, _>>()?;
            Ok(FPath(x))
        } else {
            Err(anyhow::anyhow!("Path must start with `/`"))
        }
    }
}
impl<'a> IntoIterator for FPath<'a> {
    type Item = &'a str;

    type IntoIter = std::vec::IntoIter<&'a str>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
// pub struct Iter<'a>(std::slice::Iter<'a, &'a str>);
// impl<'a> FPath<'a> {
//     pub fn iter<'b: 'a>(&'b self) -> Iter<'a> {
//         Iter(self.0.iter())
//     }
// }

// impl<'a> Iterator for Iter<'a> {
//     type Item = &'a str;

//     fn next(&mut self) -> Option<Self::Item> {
//         self.0.next().map(|p| *p)
//     }
// }
