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
        // TODO: iter_mut???
        let iter = TreeNodeIterator {
            root: self,
            current_node: None,
            current_path: TreeNodePath::empty(),
        };

        iter
    }
}

pub struct TreeNodeIterator<'a, T> {
    root: &'a TreeNode<T>,
    current_node: Option<&'a TreeNode<T>>,
    current_path: TreeNodePath,
}

impl<'a, T> Iterator for TreeNodeIterator<'a, T> {
    // type Item = (&'a TreeNode<T>, &'a TreeNodePath); // enumerate?
    type Item = &'a TreeNode<T>;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: avoid calling .get_child(). store branch: Vec<&'a TreeNode<T>> (or VecDeque?)
        // TODO: skip closed nodes? can that be achieved with .filter (is filter lazy and ordered and deterministic?)

        if let Some(current_node) = self.current_node {
            let current_node_first_child = current_node.children().first();

            if current_node_first_child.is_some() {
                self.current_path = self.current_path.with_child(0);
                self.current_node = current_node_first_child;
                current_node_first_child
            } else {
                // current_node has no children. move to next sibling, if there are more;
                // otherwise, move up, if we are not the root;
                // otherwise, we're done.

                if self.current_path.len() == 1 {
                    // no more parents. we're at the top level.
                    // we return the next sibling if there is one, None otherwise.
                    self.current_path = self.current_path.next();
                    self.current_node = self.root.children.get(self.current_path.last());
                    self.current_node
                } else {
                    // get next sibling
                    let parent_path = self.current_path.parent();
                    let parent = self.root.get_child(&parent_path).unwrap();
                    let next_sibling_path = self.current_path.next();
                    let next_sibling = parent.children.get(next_sibling_path.last());

                    if next_sibling.is_some() {
                        self.current_path = next_sibling_path;
                        self.current_node = next_sibling;
                        self.current_node
                    } else {
                        // go up. may need to go many levels!

                        self.current_path = self.current_path.parent().next();
                        self.current_node = self.root.get_child(&self.current_path);

                        self.current_node
                    }
                }
            }
        } else {
            if self.root.children.is_empty() {
                None
            } else {
                self.current_path = self.current_path.with_child(0);
                self.current_node = Some(&self.root.children[0]);
                self.current_node
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::components::TreeNode;

    #[test]
    pub fn test_1() {
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

        assert!(iter.next().is_some_and(|n| n.inner == "child 1"));
        assert!(iter.next().is_some_and(|n| n.inner == "child 2"));
        assert!(iter.next().is_some_and(|n| n.inner == "child 2 - 1"));
        assert!(iter.next().is_some_and(|n| n.inner == "child 2 - 2"));
        assert!(iter.next().is_some_and(|n| n.inner == "child 3"));
        assert!(iter.next().is_some_and(|n| n.inner == "child 3 - 1"));
        assert!(iter.next().is_some_and(|n| n.inner == "child 3 - 2"));
        assert!(iter.next().is_some_and(|n| n.inner == "child 3 - 2 - 1"));
        assert!(iter.next().is_some_and(|n| n.inner == "child 3 - 2 - 2"));
        assert!(iter.next().is_some_and(|n| n.inner == "child 3 - 3"));
        assert!(iter.next().is_some_and(|n| n.inner == "child 4"));
        assert!(iter.next().is_some_and(|n| n.inner == "child 4 - 1"));
        assert!(iter.next().is_none());
    }
}
