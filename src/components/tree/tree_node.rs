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
            .map(|(i, n)| 1 + n.total_open_children_count(&TreeNodePath::from_vec(vec![i]), &until_path))
            .sum()
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
}
