use std::slice::Iter;

use super::{TreeNode, TreeNodePath};
use crate::components::tree::tree_node::TreeNodeIterator;

/// An iterator over a list of `TreeNode`s.
/// Useful when you have [TreeNode<T>].
pub struct TreeNodeListIterator<'a, T: 'a> {
    root_iter: Iter<'a, TreeNode<T>>,
    root_index: isize,
    child_iter: Option<TreeNodeIterator<'a, T>>,
    pick_locks: bool,
}

impl<'a, T: 'a> TreeNodeListIterator<'a, T> {
    pub fn new(items: &'a [TreeNode<T>]) -> TreeNodeListIterator<'a, T> {
        let root_iter = items.iter();

        TreeNodeListIterator {
            root_iter,
            root_index: -1,
            child_iter: None,
            pick_locks: false,
        }
    }

    #[allow(unused)]
    pub fn new_thief(items: &'a [TreeNode<T>]) -> TreeNodeListIterator<'a, T> {
        let root_iter = items.iter();

        TreeNodeListIterator {
            root_iter,
            root_index: -1,
            child_iter: None,
            pick_locks: true,
        }
    }
}

impl<'a, T> Iterator for TreeNodeListIterator<'a, T> {
    type Item = (TreeNodePath, &'a TreeNode<T>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((path, node)) = self.child_iter.as_mut().and_then(|ci| ci.next()) {
            Some((path.with_parent(self.root_index as usize), node))
        } else if let Some(root_node) = self.root_iter.next() {
            self.child_iter = if self.pick_locks || root_node.is_open {
                Some(root_node.iter())
            } else {
                None
            };
            self.root_index += 1;
            Some((TreeNodePath::from_vec(vec![self.root_index as usize]), root_node))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TreeNodeListIterator;
    use crate::components::{TreeNode, TreeNodePath};

    fn create_test_tree_nodes() -> Vec<TreeNode<String>> {
        let mut root_1 = TreeNode::new("root 1".to_string());
        root_1.children = vec![TreeNode::new("root 1 - child 1".to_string())];

        let mut root_2 = TreeNode::new("root 2".to_string());
        root_2.children = vec![
            TreeNode::new("root 2 - child 1".to_string()),
            TreeNode::new("root 2 - child 2".to_string()),
        ];

        let mut root_3_child_2 = TreeNode::new("root 3 - child 2".to_string());
        root_3_child_2.children = vec![
            TreeNode::new("root 3 - child 2 - grandchild 1".to_string()),
            TreeNode::new("root 3 - child 2 - grandchild 2".to_string()),
        ];

        let mut root_3 = TreeNode::new("root 3".to_string());
        root_3.children = vec![
            TreeNode::new("root 3 - child 1".to_string()),
            root_3_child_2,
            TreeNode::new("root 3 - child 3".to_string()),
        ];

        let mut root_4 = TreeNode::new("root 4".to_string());
        root_4.children = vec![TreeNode::new("root 4 - child 1".to_string())];

        vec![root_1, root_2, root_3, root_4]
    }

    #[test]
    pub fn test_iter_thief() -> Result<(), ()> {
        let root_nodes = create_test_tree_nodes();
        let mut iter = TreeNodeListIterator::new_thief(&root_nodes);

        let mut assert_iter_eq = |expected_inner: &str, expected_path: &[usize]| {
            let Some((path, node)) = iter.next() else {
                panic!("Expected Some(...), got None");
            };
            assert_eq!(node.inner, expected_inner, "Unexpected node.inner at path '{path}'");
            assert_eq!(
                path,
                TreeNodePath::from_vec(expected_path.to_vec()),
                "Unexpected path for '{}'. Expected {expected_path:?}",
                node.inner
            );
        };

        assert_iter_eq("root 1", &[0]);
        assert_iter_eq("root 1 - child 1", &[0, 0]);
        assert_iter_eq("root 2", &[1]);
        assert_iter_eq("root 2 - child 1", &[1, 0]);
        let node = TreeNode::get_node_at_path(&TreeNodePath::from_vec(vec![1, 0]), &root_nodes);
        assert_eq!(node.unwrap().inner, "root 2 - child 1");

        assert_iter_eq("root 2 - child 2", &[1, 1]);
        assert_iter_eq("root 3", &[2]);
        assert_iter_eq("root 3 - child 1", &[2, 0]);
        assert_iter_eq("root 3 - child 2", &[2, 1]);
        assert_iter_eq("root 3 - child 2 - grandchild 1", &[2, 1, 0]);

        let node = TreeNode::get_node_at_path(&TreeNodePath::from_vec(vec![2, 1, 0]), &root_nodes);
        assert_eq!(node.unwrap().inner, "root 3 - child 2 - grandchild 1");
        assert_iter_eq("root 3 - child 2 - grandchild 2", &[2, 1, 1]);
        assert_iter_eq("root 3 - child 3", &[2, 2]);
        assert_iter_eq("root 4", &[3]);
        assert_iter_eq("root 4 - child 1", &[3, 0]);

        assert_eq!(iter.next(), None);

        Ok(())
    }

    #[test]
    pub fn test_iter() -> Result<(), ()> {
        let mut root_nodes = create_test_tree_nodes();
        root_nodes[1].is_open = false;
        let mut iter = TreeNodeListIterator::new(&root_nodes);

        let mut assert_iter_eq = |expected_inner: &str, expected_path: &[usize]| {
            let Some((path, node)) = iter.next() else {
                panic!("Expected Some(...), got None");
            };
            assert_eq!(node.inner, expected_inner, "Unexpected node.inner at path '{path}'");
            assert_eq!(
                path,
                TreeNodePath::from_vec(expected_path.to_vec()),
                "Unexpected path for '{}'. Expected {expected_path:?}",
                node.inner
            );
        };

        assert_iter_eq("root 1", &[0]);
        assert_iter_eq("root 1 - child 1", &[0, 0]);
        assert_iter_eq("root 2", &[1]);
        // assert_iter_eq("root 2 - child 1", &[1, 0]);
        // let node = TreeNode::get_node_at_path(&TreeNodePath::from_vec(vec![1, 0]), &root_nodes);
        // assert_eq!(node.unwrap().inner, "root 2 - child 1");

        // assert_iter_eq("root 2 - child 2", &[1, 1]);
        assert_iter_eq("root 3", &[2]);
        assert_iter_eq("root 3 - child 1", &[2, 0]);
        assert_iter_eq("root 3 - child 2", &[2, 1]);
        assert_iter_eq("root 3 - child 2 - grandchild 1", &[2, 1, 0]);
        let node = TreeNode::get_node_at_path(&TreeNodePath::from_vec(vec![2, 1, 0]), &root_nodes);
        assert_eq!(node.unwrap().inner, "root 3 - child 2 - grandchild 1");
        assert_iter_eq("root 3 - child 2 - grandchild 2", &[2, 1, 1]);
        assert_iter_eq("root 3 - child 3", &[2, 2]);
        assert_iter_eq("root 4", &[3]);
        assert_iter_eq("root 4 - child 1", &[3, 0]);

        assert_eq!(iter.next(), None);

        Ok(())
    }
}
