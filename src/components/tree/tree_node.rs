use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use super::TreeNodePath;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TreeNode<T> {
    pub inner: T,
    pub is_visible: bool,
    #[serde(skip)]
    pub is_match: bool,
    pub is_open: bool,
    pub children: Vec<Self>,
}

impl<T> TreeNode<T> {
    pub fn new(t: T) -> Self {
        Self {
            inner: t,
            is_visible: true,
            is_match: false,
            is_open: true,
            children: vec![],
        }
    }

    pub fn new_with_children(t: T, children: Vec<TreeNode<T>>) -> Self {
        Self {
            inner: t,
            is_visible: true,
            is_match: false,
            is_open: true,
            children,
        }
    }

    fn total_open_children_count(&self, base_path: &TreeNodePath, until_path: &TreeNodePath) -> usize {
        fn recursive_open_count<T>(nodes: &[TreeNode<T>], path: TreeNodePath, until_path: &TreeNodePath) -> usize {
            let mut count = 0;
            for (i, node) in nodes.iter().enumerate() {
                let new_path = path.with_child(i);

                if new_path > *until_path {
                    break;
                }

                count += 1;

                if !node.is_open || node.children.is_empty() {
                    continue;
                }

                count += recursive_open_count(&node.children, new_path, until_path);
            }
            count
        }

        if !self.is_open || self.children.is_empty() {
            0
        } else {
            recursive_open_count(&self.children, base_path.clone(), until_path)
        }
    }

    pub fn total_open_count(nodes: &[Self]) -> usize {
        // TODO: not this
        Self::open_count(
            nodes,
            &TreeNodePath::from_vec(vec![usize::MAX, usize::MAX, usize::MAX, usize::MAX, usize::MAX]),
        )
    }

    pub fn open_count(nodes: &[Self], until_path: &TreeNodePath) -> usize {
        nodes
            .iter()
            .take(until_path.first().saturating_add(1))
            .enumerate()
            .map(|(i, n)| 1 + n.total_open_children_count(&TreeNodePath::from_vec(vec![i]), until_path))
            .sum()
    }

    pub fn children(&self) -> &Vec<Self> {
        &self.children
    }

    pub fn get_child(&self, path: &TreeNodePath) -> Option<&Self> {
        assert!(!path.is_empty(), "path cannot be empty");

        let mut path = path.as_slice();
        let mut children = &self.children;

        loop {
            let node = children.get(path[0]);
            if node.is_none() || path.len() == 1 {
                return node;
            }
            path = &path[1..];
            children = &node.unwrap().children;
        }
    }

    pub fn get_node_at_path<'a>(path: &TreeNodePath, nodes: &'a [Self]) -> Option<&'a Self> {
        assert!(!path.is_empty(), "path cannot be empty");
        let index = path.first();

        if path.len() == 1 {
            Some(&nodes[index])
        } else {
            let p = TreeNodePath::from_vec(path.as_slice()[1..].to_vec());
            nodes[index].get_child(&p)
        }
    }

    pub fn get_node_at_path_mut(path: TreeNodePath, nodes: &mut [Self]) -> &mut Self {
        fn recursive<T>(mut path: VecDeque<usize>, nodes: &mut [TreeNode<T>]) -> &mut TreeNode<T> {
            let Some(next_level) = path.pop_front() else {
                panic!("get_node_at_path_mut panic");
            };

            if path.is_empty() {
                &mut nodes[next_level]
            } else {
                recursive(path, &mut nodes[next_level].children)
            }
        }
        recursive(path.to_vec().into(), nodes)
    }

    pub fn iter(&self) -> TreeNodeIterator<T> {
        TreeNodeIterator {
            root: self,
            current_node: None,
            current_path: TreeNodePath::empty(),
        }
    }

    /// Walks the entire tree, depth-first, calling the passed callback with each element.
    /// Skips _closed_ nodes (and all of their children).
    pub fn for_each_mut(nodes: &mut [TreeNode<T>], mut cb: impl FnMut(&mut TreeNode<T>, TreeNodePath)) {
        fn recursive<T>(nodes: &mut [TreeNode<T>], path: TreeNodePath, cb: &mut impl FnMut(&mut TreeNode<T>, TreeNodePath)) {
            for (index, node) in nodes.iter_mut().enumerate() {
                let path = path.with_child(index);
                cb(node, path.clone());

                if node.is_open {
                    recursive(&mut node.children, path, cb);
                }
            }
        }

        recursive(nodes, TreeNodePath::from_vec(vec![]), &mut cb);
    }

}

pub struct TreeNodeIterator<'a, T> {
    root: &'a TreeNode<T>,
    current_node: Option<&'a TreeNode<T>>,
    current_path: TreeNodePath,
}

impl<'a, T> Iterator for TreeNodeIterator<'a, T> {
    type Item = (TreeNodePath, &'a TreeNode<T>);

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: avoid calling .get_child(). store branch: Vec<&'a TreeNode<T>> (or VecDeque?)
        // TODO: skip closed nodes?

        if let Some(current_node) = self.current_node {
            let current_node_first_child = current_node.children().first();

            if current_node_first_child.is_some() {
                self.current_path = self.current_path.with_child(0);
                self.current_node = current_node_first_child;
                self.current_node.as_ref().map(|n| (self.current_path.clone(), *n))
            } else {
                // current_node has no children. move to next sibling, if there are more;
                // otherwise, move up, if we are not the root;
                // otherwise, we're done.

                if self.current_path.len() == 1 {
                    // no more parents. we're at the top level.
                    // we return the next sibling if there is one, None otherwise.
                    self.current_path = self.current_path.next_sibling();
                    self.current_node = self.root.children.get(self.current_path.clone().last());
                    self.current_node.as_ref().map(|n| (self.current_path.clone(), *n))
                } else {
                    // get next sibling
                    let parent_path = self.current_path.parent();
                    let parent = self.root.get_child(&parent_path).unwrap();
                    let next_sibling_path = self.current_path.next_sibling();
                    let next_sibling = parent.children.get(next_sibling_path.last());

                    if next_sibling.is_some() {
                        self.current_path = next_sibling_path;
                        self.current_node = next_sibling;
                        self.current_node.as_ref().map(|n| (self.current_path.clone(), *n))
                    } else {
                        // go up. may need to go many levels!

                        self.current_path = self.current_path.parent().next_sibling();
                        self.current_node = self.root.get_child(&self.current_path);

                        self.current_node.as_ref().map(|n| (self.current_path.clone(), *n))
                    }
                }
            }
        } else if self.root.children.is_empty() {
            None
        } else {
            self.current_path = self.current_path.with_child(0);
            self.current_node = Some(&self.root.children[0]);
            self.current_node.as_ref().map(|n| (self.current_path.clone(), *n))
            // Some((self.current_path.clone(), self.current_node.unwrap()))
        }
    }
}

impl<T> DoubleEndedIterator for TreeNodeIterator<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.root.children.is_empty() {
            None
        } else if self.current_node.is_none() {
            // We're just getting started. Find the very last node and return it.
            let mut node = self.root;

            loop {
                if let Some(child) = node.children.last() {
                    self.current_path = self.current_path.with_child(node.children.len() - 1);
                    node = child;
                } else {
                    self.current_node = Some(node);
                    break self.current_node.as_ref().map(|n| (self.current_path.clone(), *n));
                }
            }
        } else if self.current_path.is_empty() || self.current_path[0] == 0 {
            // Can't proceed any further. We're done.
            None
        } else if self.current_path.last() == 0 {
            // We're the last child. Move upwards, return parent.
            self.current_path = self.current_path.parent();
            self.current_node = self.root.get_child(&self.current_path);
            self.current_node.as_ref().map(|n| (self.current_path.clone(), *n))
        } else {
            // We have at least one sibling before us. Move onto it. (self.current_path.last() > 0)
            self.current_path = self.current_path.prev_sibling().unwrap();
            let mut node = self.root.get_child(&self.current_path).unwrap();

            // This sibling may have children. We now must find its last child.
            loop {
                if let Some(child) = node.children.last() {
                    self.current_path = self.current_path.with_child(node.children.len() - 1);
                    node = child;
                } else {
                    self.current_node = Some(node);
                    break self.current_node.as_ref().map(|n| (self.current_path.clone(), *n));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::components::{TreeNode, TreeNodePath};

    #[test]
    pub fn test_1() -> Result<(), ()> {
        let mut child_2 = TreeNode::new("child 2");
        child_2.children = vec![TreeNode::new("child 2 - 1"), TreeNode::new("child 2 - 2")];

        let mut child_3_2 = TreeNode::new("child 3 - 2");
        child_3_2.children = vec![TreeNode::new("child 3 - 2 - 1"), TreeNode::new("child 3 - 2 - 2")];

        let mut child_3 = TreeNode::new("child 3");
        child_3.children = vec![TreeNode::new("child 3 - 1"), child_3_2, TreeNode::new("child 3 - 3")];

        let mut child_4 = TreeNode::new("child 4");
        child_4.children = vec![TreeNode::new("child 4 - 1")];

        let mut root = TreeNode::new("root");
        root.children = vec![TreeNode::new("child 1"), child_2, child_3, child_4];

        let mut iter = root.iter();

        let mut assert_iter_eq = |expected_inner: &str, expected_path: &[usize]| {
            let Some((path, node)) = iter.next() else {
                panic!("Expected Some(...), got None");
            };
            assert_eq!(node.inner, expected_inner, "Unexpected node.inner at path '{path}'");
            assert_eq!(
                path,
                TreeNodePath::from_vec(expected_path.to_vec()),
                "Unexpected path for '{}'",
                node.inner
            );
        };

        assert_iter_eq("child 1", &[0]);
        assert_iter_eq("child 2", &[1]);
        assert_iter_eq("child 2 - 1", &[1, 0]);
        assert_iter_eq("child 2 - 2", &[1, 1]);
        assert_iter_eq("child 3", &[2]);
        assert_iter_eq("child 3 - 1", &[2, 0]);
        assert_iter_eq("child 3 - 2", &[2, 1]);
        assert_iter_eq("child 3 - 2 - 1", &[2, 1, 0]);
        assert_iter_eq("child 3 - 2 - 2", &[2, 1, 1]);
        assert_iter_eq("child 3 - 3", &[2, 2]);
        assert_iter_eq("child 4", &[3]);
        assert_iter_eq("child 4 - 1", &[3, 0]);
        assert!(iter.next().is_none());

        Ok(())
    }

    #[test]
    pub fn test_1_rev() -> Result<(), ()> {
        let mut child_2 = TreeNode::new("child 2");
        child_2.children = vec![TreeNode::new("child 2 - 1"), TreeNode::new("child 2 - 2")];

        let mut child_3_2 = TreeNode::new("child 3 - 2");
        child_3_2.children = vec![TreeNode::new("child 3 - 2 - 1"), TreeNode::new("child 3 - 2 - 2")];

        let mut child_3 = TreeNode::new("child 3");
        child_3.children = vec![TreeNode::new("child 3 - 1"), child_3_2, TreeNode::new("child 3 - 3")];

        let mut child_4 = TreeNode::new("child 4");
        child_4.children = vec![TreeNode::new("child 4 - 1")];

        let mut root = TreeNode::new("root");
        root.children = vec![TreeNode::new("child 1"), child_2, child_3, child_4];

        let mut iter = root.iter().rev();

        let mut assert_iter_eq = |expected_inner: &str, expected_path: &[usize]| {
            let Some((path, node)) = iter.next() else {
                panic!("Expected Some(...), got None");
            };
            assert_eq!(node.inner, expected_inner, "Unexpected node.inner at path '{path}'");
            assert_eq!(
                path,
                TreeNodePath::from_vec(expected_path.to_vec()),
                "Unexpected path for '{}'",
                node.inner
            );
        };

        assert_iter_eq("child 4 - 1", &[3, 0]);
        assert_iter_eq("child 4", &[3]);
        assert_iter_eq("child 3 - 3", &[2, 2]);
        assert_iter_eq("child 3 - 2 - 2", &[2, 1, 1]);
        assert_iter_eq("child 3 - 2 - 1", &[2, 1, 0]);
        assert_iter_eq("child 3 - 2", &[2, 1]);
        assert_iter_eq("child 3 - 1", &[2, 0]);
        assert_iter_eq("child 3", &[2]);
        assert_iter_eq("child 2 - 2", &[1, 1]);
        assert_iter_eq("child 2 - 1", &[1, 0]);
        assert_iter_eq("child 2", &[1]);
        assert_iter_eq("child 1", &[0]);
        assert!(iter.next().is_none());

        Ok(())
    }
}
