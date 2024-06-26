use std::{
    cmp::{max, Ordering},
    iter::FromIterator,
    mem,
    ops::Not,
};

/// An internal node of an `AVLTree`.
struct AVLNode<T: Ord> {
    value: T,
    height: usize,
    left: Option<Box<AVLNode<T>>>,
    right: Option<Box<AVLNode<T>>>,
}

/// An AVL tree.
///
/// An AVL tree is a self-balancing binary search tree where the height difference between the left
/// and right subtrees of each node is at most 1. This guarantees a logarithmic height and
/// efficient operations.
///
/// # Examples
///
/// ```rust
/// use rust_algorithms::data_structures::AVLTree;
///
/// let mut tree = AVLTree::new();
/// tree.insert(1);
/// tree.insert(2);
///
/// assert!(tree.contains(&1));
/// assert!(tree.contains(&2));
/// assert!(!tree.contains(&3));
/// ```
pub struct AVLTree<T: Ord> {
    root: Option<Box<AVLNode<T>>>,
    length: usize,
}

/// Refers to the left or right subtree of an `AVLNode`.
#[derive(Clone, Copy)]
enum Side {
    Left,
    Right,
}

impl<T: Ord> AVLTree<T> {
    /// Creates an empty `AVLTree`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_algorithms::data_structures::AVLTree;
    ///
    /// let tree: AVLTree<i32> = AVLTree::new();
    ///
    /// assert!(tree.is_empty());
    /// ```
    pub fn new() -> AVLTree<T> {
        AVLTree {
            root: None,
            length: 0,
        }
    }

    /// Checks if the tree contains a value.
    ///
    /// # Arguments
    ///
    /// * `value`: A reference to the value to check for.
    ///
    /// # Returns
    ///
    /// `true` if the tree contains the value, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_algorithms::data_structures::AVLTree;
    ///
    /// let mut tree = AVLTree::new();
    /// tree.insert(1);
    ///
    /// // The tree should contain the value 1.
    /// assert!(tree.contains(&1));
    /// // The tree should not contain the value 2.
    /// assert!(!tree.contains(&2));
    /// ```
    pub fn contains(&self, value: &T) -> bool {
        let mut current = &self.root;
        while let Some(node) = current {
            current = match value.cmp(&node.value) {
                Ordering::Equal => return true,
                Ordering::Less => &node.left,
                Ordering::Greater => &node.right,
            }
        }
        false
    }

    /// Adds a value to the tree.
    ///
    /// # Returns
    ///
    /// `true` if the tree did not yet contain the value, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_algorithms::data_structures::AVLTree;
    ///
    /// let mut tree = AVLTree::new();
    ///
    /// // The first insertion should succeed.
    /// assert!(tree.insert(1));
    /// // The second insertion should fail, since the value is already in the tree.
    /// assert!(!tree.insert(1));
    /// ```
    pub fn insert(&mut self, value: T) -> bool {
        let inserted = insert(&mut self.root, value);
        if inserted {
            self.length += 1;
        }
        inserted
    }

    /// Removes a value from the tree.
    ///
    /// # Returns
    ///
    /// `true` if the tree contained the value, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_algorithms::data_structures::AVLTree;
    ///
    /// let mut tree = AVLTree::new();
    /// tree.insert(1);
    ///
    /// // First removal should succeed, since the value is in the tree.
    /// assert!(tree.remove(&1));
    /// // The second removal should fail, since the value is no longer in the tree.
    /// assert!(!tree.remove(&1));
    /// ```
    pub fn remove(&mut self, value: &T) -> bool {
        let removed = remove(&mut self.root, value);
        if removed {
            self.length -= 1;
        }
        removed
    }

    /// Returns the number of values in the tree.
    ///
    /// # Returns
    ///
    /// The number of values in the tree.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_algorithms::data_structures::AVLTree;
    ///
    /// let tree: AVLTree<_> = (1..4).collect();
    ///
    /// assert_eq!(tree.len(), 3);
    /// ```
    pub fn len(&self) -> usize {
        self.length
    }

    /// Detects if the AVL Tree is empty.
    ///
    /// # Returns
    ///
    /// `true` if the tree contains no values, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_algorithms::data_structures::AVLTree;
    ///
    /// let tree: AVLTree<i32> = AVLTree::new();
    ///
    /// assert!(tree.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// Returns an iterator that visits the nodes in the tree in order.
    fn node_iter(&self) -> NodeIter<T> {
        let cap = self.root.as_ref().map_or(0, |n| n.height);
        let mut node_iter = NodeIter {
            stack: Vec::with_capacity(cap),
        };
        // Initialize stack with path to leftmost child
        let mut child = &self.root;
        while let Some(node) = child {
            node_iter.stack.push(node.as_ref());
            child = &node.left;
        }
        node_iter
    }

    /// Gets an iterator that visits the values in the tree in ascending order.
    ///
    /// # Returns
    ///
    /// An iterator that visits the values in the tree in ascending order.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_algorithms::data_structures::AVLTree;
    ///
    /// let tree: AVLTree<_> = (1..4).collect();
    ///
    /// assert_eq!(tree.iter().collect::<Vec<_>>(), vec![&1, &2, &3]);
    /// ```
    pub fn iter(&self) -> Iter<T> {
        Iter {
            node_iter: self.node_iter(),
        }
    }
}

/// Recursive helper function for `AVLTree` insertion.
fn insert<T: Ord>(tree: &mut Option<Box<AVLNode<T>>>, value: T) -> bool {
    if let Some(node) = tree {
        let inserted = match value.cmp(&node.value) {
            Ordering::Equal => false,
            Ordering::Less => insert(&mut node.left, value),
            Ordering::Greater => insert(&mut node.right, value),
        };
        if inserted {
            node.rebalance();
        }
        inserted
    } else {
        *tree = Some(Box::new(AVLNode {
            value,
            height: 1,
            left: None,
            right: None,
        }));
        true
    }
}

/// Recursive helper function for `AVLTree` deletion.
fn remove<T: Ord>(tree: &mut Option<Box<AVLNode<T>>>, value: &T) -> bool {
    if let Some(node) = tree {
        let removed = match value.cmp(&node.value) {
            Ordering::Less => remove(&mut node.left, value),
            Ordering::Greater => remove(&mut node.right, value),
            Ordering::Equal => {
                *tree = match (node.left.take(), node.right.take()) {
                    (None, None) => None,
                    (Some(b), None) | (None, Some(b)) => Some(b),
                    (Some(left), Some(right)) => Some(merge(left, right)),
                };
                return true;
            }
        };
        if removed {
            node.rebalance();
        }
        removed
    } else {
        false
    }
}

/// Merges two trees and returns the root of the merged tree.
fn merge<T: Ord>(left: Box<AVLNode<T>>, right: Box<AVLNode<T>>) -> Box<AVLNode<T>> {
    let mut op_right = Some(right);
    // Guaranteed not to panic since right has at least one node
    let mut root = take_min(&mut op_right).unwrap();
    root.left = Some(left);
    root.right = op_right;
    root.rebalance();
    root
}

/// Removes the smallest node from the tree, if one exists.
fn take_min<T: Ord>(tree: &mut Option<Box<AVLNode<T>>>) -> Option<Box<AVLNode<T>>> {
    if let Some(mut node) = tree.take() {
        // Recurse along the left side
        if let Some(small) = take_min(&mut node.left) {
            // Took the smallest from below; update this node and put it back in the tree
            node.rebalance();
            *tree = Some(node);
            Some(small)
        } else {
            // Take this node and replace it with its right child
            *tree = node.right.take();
            Some(node)
        }
    } else {
        None
    }
}

impl<T: Ord> AVLNode<T> {
    /// Returns a reference to the left or right child.
    fn child(&self, side: Side) -> &Option<Box<AVLNode<T>>> {
        match side {
            Side::Left => &self.left,
            Side::Right => &self.right,
        }
    }

    /// Returns a mutable reference to the left or right child.
    fn child_mut(&mut self, side: Side) -> &mut Option<Box<AVLNode<T>>> {
        match side {
            Side::Left => &mut self.left,
            Side::Right => &mut self.right,
        }
    }

    /// Returns the height of the left or right subtree.
    fn height(&self, side: Side) -> usize {
        self.child(side).as_ref().map_or(0, |n| n.height)
    }

    /// Returns the height difference between the left and right subtrees.
    fn balance_factor(&self) -> i8 {
        let (left, right) = (self.height(Side::Left), self.height(Side::Right));
        if left < right {
            (right - left) as i8
        } else {
            -((left - right) as i8)
        }
    }

    /// Recomputes the `height` field.
    fn update_height(&mut self) {
        self.height = 1 + max(self.height(Side::Left), self.height(Side::Right));
    }

    /// Performs a left or right rotation.
    fn rotate(&mut self, side: Side) {
        let mut subtree = self.child_mut(!side).take().unwrap();
        *self.child_mut(!side) = subtree.child_mut(side).take();
        self.update_height();
        // Swap root and child nodes in memory
        mem::swap(self, subtree.as_mut());
        // Set old root (subtree) as child of new root (self)
        *self.child_mut(side) = Some(subtree);
        self.update_height();
    }

    /// Performs left or right tree rotations to balance this node.
    fn rebalance(&mut self) {
        self.update_height();
        let side = match self.balance_factor() {
            -2 => Side::Left,
            2 => Side::Right,
            _ => return,
        };
        let subtree = self.child_mut(side).as_mut().unwrap();
        // Left-Right and Right-Left require rotation of heavy subtree
        if let (Side::Left, 1) | (Side::Right, -1) = (side, subtree.balance_factor()) {
            subtree.rotate(side);
        }
        // Rotate in opposite direction of heavy side
        self.rotate(!side);
    }
}

/// Default implementation for `AVLTree`.
///
/// Creates an empty `AVLTree`.
impl<T: Ord> Default for AVLTree<T> {
    /// Creates an empty `AVLTree`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_algorithms::data_structures::AVLTree;
    ///
    /// let tree: AVLTree<i32> = Default::default();
    ///
    /// assert!(tree.is_empty());
    /// ```
    fn default() -> Self {
        Self::new()
    }
}

impl Not for Side {
    type Output = Side;

    fn not(self) -> Self::Output {
        match self {
            Side::Left => Side::Right,
            Side::Right => Side::Left,
        }
    }
}

impl<T: Ord> FromIterator<T> for AVLTree<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut tree = AVLTree::new();
        for value in iter {
            tree.insert(value);
        }
        tree
    }
}

/// An iterator over the nodes of an `AVLTree`.
///
/// This struct is created by the `node_iter` method of `AVLTree`.
struct NodeIter<'a, T: Ord> {
    stack: Vec<&'a AVLNode<T>>,
}

/// An iterator over the nodes of an `AVLTree`.
///
/// This struct is created by the `node_iter` method of `AVLTree`.
impl<'a, T: Ord> Iterator for NodeIter<'a, T> {
    type Item = &'a AVLNode<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.stack.pop() {
            // Push left path of right subtree to stack
            let mut child = &node.right;
            while let Some(subtree) = child {
                self.stack.push(subtree.as_ref());
                child = &subtree.left;
            }
            Some(node)
        } else {
            None
        }
    }
}

/// An iterator over the items of an `AVLTree`.
///
/// This struct is created by the `iter` method of `AVLTree`.
pub struct Iter<'a, T: Ord> {
    node_iter: NodeIter<'a, T>,
}

/// An iterator over the items of an `AVLTree`.
///
/// This struct is created by the `iter` method of `AVLTree`.
impl<'a, T: Ord> Iterator for Iter<'a, T> {
    type Item = &'a T;

    /// Returns the next value in the tree.
    ///
    /// # Returns
    ///
    /// The next value in the tree, or `None` if there are no more values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rust_algorithms::data_structures::AVLTree;
    ///
    /// let tree: AVLTree<_> = (1..4).collect();
    /// let mut iter = tree.iter();
    ///
    /// assert_eq!(iter.next(), Some(&1));
    /// assert_eq!(iter.next(), Some(&2));
    /// assert_eq!(iter.next(), Some(&3));
    /// assert_eq!(iter.next(), None);
    /// ```
    fn next(&mut self) -> Option<&'a T> {
        match self.node_iter.next() {
            Some(node) => Some(&node.value),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AVLTree;

    /// Returns `true` if all nodes in the tree are balanced.
    fn is_balanced<T: Ord>(tree: &AVLTree<T>) -> bool {
        tree.node_iter()
            .all(|n| (-1..=1).contains(&n.balance_factor()))
    }

    #[test]
    fn sorted() {
        let tree: AVLTree<_> = (1..8).rev().collect();
        assert!((1..8).eq(tree.iter().map(|&x| x)));
    }

    #[test]
    fn balanced() {
        let mut tree: AVLTree<_> = (1..8).collect();
        assert!(is_balanced(&tree));
        for x in 1..8 {
            tree.remove(&x);
            assert!(is_balanced(&tree));
        }
    }
}
