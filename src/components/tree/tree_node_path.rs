use std::{
    fmt::{Display, Formatter},
    ops::Index,
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct TreeNodePath(Vec<usize>);

impl TreeNodePath {
    pub fn empty() -> Self {
        Self(vec![])
    }

    pub fn zero() -> Self {
        Self(vec![0])
    }

    pub fn from_vec(vec: Vec<usize>) -> Self {
        Self(vec)
    }

    pub fn as_slice(&self) -> &[usize] {
        self.0.as_slice()
    }

    pub fn to_vec(&self) -> Vec<usize> {
        self.0.clone()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn first(&self) -> usize {
        self.0[0]
    }

    pub fn last(&self) -> usize {
        self[self.len().saturating_sub(1)]
    }

    pub fn parent(&self) -> Self {
        let mut parent = self.clone();
        let new_len = parent.len().saturating_sub(1);
        parent.0.truncate(new_len);
        parent
    }

    pub fn with_child(&self, i: usize) -> Self {
        let mut path = self.clone();
        path.0.push(i);
        path
    }

    pub fn with_value(&self, index: usize, value: usize) -> Self {
        let mut new = self.clone();
        new.0[index] = value;
        new
    }

    pub fn without_first(&self) -> Self {
        TreeNodePath::from_vec(self.0.as_slice()[1..].to_vec())
    }
}

impl Index<usize> for TreeNodePath {
    type Output = usize;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
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
