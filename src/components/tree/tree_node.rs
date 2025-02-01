use std::{cmp::Ordering, collections::VecDeque};

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

    pub fn total_open_count(nodes: &[TreeNode<T>]) -> usize {
        nodes.len() + nodes.iter().map(|node| node.total_open_children_count()).sum::<usize>()
    }

    pub fn open_count(nodes: &[TreeNode<T>], until_path: &TreeNodePath) -> usize {
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

    pub fn get_node_at_path(path: TreeNodePath, nodes: &[TreeNode<T>]) -> &TreeNode<T> {
        assert!(!path.is_empty(), "path cannot be empty");
        fn recursive<'a, T>(mut path: &[usize], nodes: &'a [TreeNode<T>]) -> &'a TreeNode<T> {
            let index = path[0];
            let path = &path[1..];

            if path.is_empty() {
                &nodes[index]
            } else {
                recursive(path, &nodes[index].children)
            }
        }
        recursive(path.as_slice(), nodes)
    }

    pub fn get_node_at_path_mut(path: TreeNodePath, nodes: &mut [TreeNode<T>]) -> &mut TreeNode<T> {
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
