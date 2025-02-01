use std::{cmp::Ordering, collections::VecDeque, ops::Index};

use super::TreeNodePath;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreeNode<T> {
    pub inner: T,
    pub is_visible: bool,
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

    fn total_open_children_count(&self) -> usize {
        fn recursive_total_open_count<T>(nodes: &[TreeNode<T>]) -> usize {
            let mut count = 0;
            for i in 0..nodes.len() {
                count += 1;

                if !nodes[i].is_open || nodes[i].children.is_empty() {
                    continue;
                }

                count += recursive_total_open_count(&nodes[i].children);
            }
            count
        }
        if !self.is_open || self.children.is_empty() {
            0
        } else {
            recursive_total_open_count(&self.children)
        }
    }

    pub fn total_open_count(nodes: &[Self]) -> usize {
        nodes.len() + nodes.iter().map(|node| node.total_open_children_count()).sum::<usize>()
    }

    pub fn open_count(nodes: &[Self], until_path: &TreeNodePath) -> usize {
        fn recursive_open_count<T>(nodes: &[TreeNode<T>], path: TreeNodePath, until_path: &TreeNodePath) -> usize {
            let mut count = 0;
            for i in 0..nodes.len() {
                let new_path = path.with_child(i);

                if new_path.cmp(until_path) >= Ordering::Equal {
                    break;
                }

                count += 1;

                if !nodes[i].is_open || nodes[i].children.is_empty() {
                    continue;
                }

                count += recursive_open_count(&nodes[i].children, new_path, until_path);
            }
            count
        }
        recursive_open_count(nodes, TreeNodePath::empty(), until_path)
    }

    pub fn get_child(&self, path: &TreeNodePath) -> Option<&Self> {
        assert!(!path.is_empty(), "path cannot be empty");

        fn recursive<'a, T>(path: &[usize], nodes: &'a [TreeNode<T>]) -> Option<&'a TreeNode<T>> {
            let node = nodes.get(path[0]);

            if node.is_some() && path.len() == 1 {
                node
            } else {
                node.and_then(|node| {
                    recursive(&path[1..], &node.children)
                })
            }

        }

        recursive(path.as_slice(), &self.children)
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
}
