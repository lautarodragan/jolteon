use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd)]
pub struct TreeNodePath(pub Vec<usize>);

impl TreeNodePath {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn parent(&self) -> Self {
        let mut parent = self.clone();
        let new_len = parent.len().saturating_sub(1);
        parent.truncate(new_len);
        parent
    }

    pub fn deepest(&self) -> usize {
        self[self.len().saturating_sub(1)]
    }

    pub fn with_child(&self, i: usize) -> Self {
        let mut path = self.clone();
        path.push(i);
        path
    }
}

impl Deref for TreeNodePath {
    type Target = Vec<usize>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TreeNodePath {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Ord for TreeNodePath {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut j = 0;
        loop {
            if j >= self.0.len().min(other.0.len()) {
                break self.0.len().cmp(&other.0.len());
            }

            let ord = self.0[j].cmp(&other.0[j]);

            if ord != Ordering::Equal {
                break ord;
            }

            j += 1;
        }
    }
}

impl Display for TreeNodePath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for i in &self.0 {
            write!(f, "/{i}")?;
        }
        Ok(())
    }
}
