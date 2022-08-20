//! `feap` is an implementation of a 
//! [Fibonacci Heap](https://en.wikipedia.org/wiki/Fibonacci_heap) and designed
//! to be fast. It is generic and only [`PartialOrd`] has to be implemented.
//! 
//! Example:
//! 
//! ```rust
//! let mut feap = FibHeap::new();
//! feap.insert(10);
//! assert_eq!(feap.get_min(), Some(&10));
//! assert_eq!(feap.extract_min(), Some(10));
//! assert_eq!(feap.extract_min(), None);
//! ```

use core::ptr;

/// The maximum allowed degree of a tree.
const MAX_DEGREE: usize = 0x100;

/// If the [`head_list`](FibHeap::head_list) is too large, consolidating takes a
/// long time. To prevent that if the [`head_list`](FibHeap::head_list) includes
/// more nodes than [`CONSOLIDATION_THRESHOLD`] than a consolidation will happen
/// even on inserts.
const CONSOLIDATION_THRESHOLD: usize = 100;

/// Wrapper type around a mutable reference to a [`Node`].
type Link<T> = *mut Node<T>;

/// A node in the tree which holds the actual value, links to its parent and
/// children and additional information of the node.
#[derive(Debug)]
struct Node<T> {
    /// A pointer to the parent node of this node.
    parent: Link<T>,

    /// A list of pointers to all of the children of this node.
    children: Vec<Link<T>>,

    /// The degree of this node. The degree tells how deep the tree is at max.
    degree: u8,

    /// In order to keep the number of children in relation to the degree of the
    /// tree in check occasionally a node has to be cut out of the tree, because
    /// the tree lost too many grandchildren. `marked` keeps track of whether
    /// this node has lost a child already.
    marked: bool,

    /// The value of this node. The value can only be accessed via the 
    /// [`get_min`](FibHeap::get_min) or [`extract_min`](FibHeap::extract_min)
    /// methods, and not directly accessed, because if it can be changed, we
    /// may lose the heap property.
    val: T
}

impl<T: PartialOrd> Node<T> {
    fn new(val: T) -> Self {
        Self {
            parent:   core::ptr::null_mut(),
            children: Vec::new(),
            degree:   0,
            marked:   false,
            val
        }
    }
}

/// The actual fibonacci heap structure.
#[derive(Clone)]
pub struct FibHeap<T: PartialOrd> {
    /// A pointer to the current minimum for convenient and faster access.
    min: Link<T>,

    /// The list of all the roots of trees currently in this fibonacci heap.
    head_list: Vec<Link<T>>,

    /// A list to temporarily save new roots during consolidation.
    root_list: Vec<Link<T>>,
}

impl<T: PartialOrd> Drop for FibHeap<T> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T: PartialOrd> Default for FibHeap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: PartialOrd> FibHeap<T> {
    /// Create a new [`FibHeap`] object. The lists are preallocated with some
    /// capacity to save on some ms for not needing to call `realloc`.
    /// 
    /// ```rust
    /// use feap::FibHeap;
    /// let feap = FibHeap::<i32>::new();
    /// ```
    pub fn new() -> Self {
        Self {
            min: ptr::null_mut(),
            head_list: Vec::with_capacity(CONSOLIDATION_THRESHOLD),
            root_list: vec![ptr::null_mut(); MAX_DEGREE],
        }
    }

    /// The recursive clearing method which drops the references to the children
    /// of a given node.
    fn _clear(&self, node: Link<T>) {
        unsafe {
            let children = &(*node).children;
            for &c in children {
                self._clear(c);
            }
            drop(Box::from_raw(node));
        }
    }

    /// [`clear`](FibHeap::clear) will deallocate all nodes in the heap and 
    /// reset the [`head_list`](FibHeap::head_list) as well as the 
    /// [`min`](FibHeap::min).
    /// 
    /// ```rust
    /// use feap::FibHeap;
    ///
    /// let mut feap = FibHeap::new();
    /// feap.insert(10);
    /// feap.clear();
    /// 
    /// assert_eq!(feap.get_min(), None);
    /// ```
    pub fn clear(&mut self) {
        for &t in &self.head_list {
            self._clear(t);
        }
        self.head_list.clear();
        self.min = ptr::null_mut();
    }

    /// `get_min` returns an immutable reference to the value of the minimum if
    /// the heap has any nodes inside. If not, `None` is returned.
    /// 
    /// ```rust
    /// use feap::FibHeap;
    ///
    /// let mut feap = FibHeap::new();
    /// 
    /// feap.insert(11);
    /// feap.insert(10);
    /// assert_eq!(feap.get_min(), Some(&10));
    /// feap.insert(4);
    /// assert_eq!(feap.get_min(), Some(&4));
    /// ```
    pub fn get_min(&self) -> Option<&T> {
        unsafe { self.min.as_ref().map(|m| &m.val) }
    }

    /// `insert` will create a new node with the value and insert it into the
    /// [`head_list`](FibHeap::head_list) (updating the [`min`](FibHeap::min)
    /// if needed). Additionally to keep the [`head_list`](FibHeap::head_list)
    /// small, if the length of the [`head_list`](FibHeap::head_list) becomes
    /// larger than the [`CONSOLIDATION_THRESHOLD`] a consolidation will happen.
    /// 
    /// ```rust
    /// use feap::FibHeap;
    ///
    /// let mut feap = FibHeap::new();
    /// 
    /// feap.insert(10);
    /// assert_eq!(feap.get_min(), Some(&10));
    /// ```
    pub fn insert(&mut self, val: T) {
        let new = Box::into_raw(Box::new(Node::new(val)));
        self.insert_node(new);
        if self.head_list.len() > CONSOLIDATION_THRESHOLD {
            self.consolidate(true);
        }
    }

    /// An internal helper function which updates the minimum if necessary and
    /// insert a node into the [`head_list`](FibHeap::head_list).
    fn insert_node(&mut self, new: Link<T>) {
        unsafe {
            if self.min.is_null() || (*new).val < (*self.min).val {
                self.min = new;
            }
            self.head_list.push(new);
        }
    }

    /// `consolidate` will firstly (if called from 
    /// [`extract_min`](FibHeap::extract_min)) put all children from the current
    /// [`min`](FibHeap::min) node into the [`head_list`](FibHeap::head_list).
    /// Then all trees in the [`head_list`](FibHeap::head_list) will be (if 
    /// possible) paired with another tree of the same [`degree`](Node::degree).
    /// If there is such a pair, the one with the greater value of the two will 
    /// become a child of the one with the smaller value, which causes the 
    /// [`degree`](Node::degree) of the tree with the smaller value to increase
    /// by one. Lastly all remaining root nodes will become the new 
    /// [`head_list`](FibHeap::head_list).
    fn consolidate(&mut self, insert_mode: bool) {
        unsafe {
            if self.min.is_null() {
                return;
            }

            // Remove all children from min if we are not doing an insert
            if !insert_mode {
                for &c in &(*self.min).children {
                    self.head_list.push(c);
                    (*c).parent = core::ptr::null_mut();
                }
            }

            // Merge trees
            self.root_list.clear();
            self.root_list.resize(MAX_DEGREE, ptr::null_mut());
            for &c in &self.head_list {
                if insert_mode || c != self.min {
                    let mut tmp = insert_root_list(c, &mut self.root_list);
                    while !tmp.is_null() {
                        tmp = insert_root_list(tmp, &mut self.root_list);
                    }
                }
            }

            // Update head_list
            self.min = ptr::null_mut();
            self.head_list.clear();

            for &n in &self.root_list {
                if !n.is_null() {
                    if self.min.is_null() || (*n).val < (*self.min).val {
                        self.min = n;
                    }
                    self.head_list.push(n);
                }
            }
        }
    }

    /// `extract_min` returns the value of the current minimum. This deallocates
    /// the current minimum node, causing a consolidation of the tree.
    /// 
    /// ```rust
    /// use feap::FibHeap;
    /// 
    /// let mut feap = FibHeap::new();
    /// 
    /// feap.insert(10);
    /// feap.insert(4);
    /// assert_eq!(feap.extract_min(), Some(4));
    /// assert_eq!(feap.extract_min(), Some(10));
    /// assert_eq!(feap.extract_min(), None);
    /// ```
    pub fn extract_min(&mut self) -> Option<T> {
        unsafe {
            if self.min.is_null() {
                return None;
            }

            let ret = self.min;

            self.consolidate(false);

            Some(Box::from_raw(ret).val)
        }
    }

    /// `find_elem` is a helper function, which traverses a tree, trying to find
    /// a node with a given value.
    fn find_elem(&self, cur_node: Link<T>, val: &T) -> Option<Link<T>> {
        unsafe {
            for &c in &(*cur_node).children {
                if (*c).val.eq(val) {
                    return Some(c)
                } else if (*c).val.lt(val) {
                    return self.find_elem(c, val);
                }
            }
            None
        }
    }

    /// `cut_out` is a function, which cuts out a sub tree from a tree and if
    /// the parent of the subtree has been marked already also cut out that
    /// node.
    fn cut_out(&mut self, node: Link<T>) {
        unsafe {
            (*node).marked = false;
            if !(*node).parent.is_null() {
                let parent = (*node).parent;
                self.insert_node(node);
                let idx = (*parent).children.iter()
                    .position(|&v| (*v).val == (*node).val)
                    .unwrap();
                (*parent).children.remove(idx);
                if !(*parent).marked {
                    (*parent).marked = true;
                } else {
                    self.cut_out(parent);
                }
            }
        }
    }

    /// `decrease_key` looks for a node with the value `old_val` and changes it
    /// to `new_val`. If the new value would invalidate the heap property, the
    /// node will be cut out.
    /// 
    /// ```rust
    /// use feap::FibHeap;
    /// 
    /// let mut feap = FibHeap::new();
    /// feap.insert(5);
    /// feap.insert(10);
    /// assert_eq!(feap.get_min(), Some(&5));
    /// feap.decrease_key(10, 3);
    /// assert_eq!(feap.get_min(), Some(&3));
    /// ```
    pub fn decrease_key(&mut self, old_val: T, new_val: T) {
        unsafe {
            let mut cur_node = None;
            for &t in &self.head_list {
                cur_node = self.find_elem(t, &old_val);
                if cur_node.is_some() { break; }
            }
    
            if let Some(cur_node) = cur_node {
                let parent = (*cur_node).parent;
                if !parent.is_null() && (*parent).val >= new_val {
                    self.cut_out(cur_node);
                }
            }
        }
    }
}

/// `insert_root_list` is a helper, that inserts a node into a root_list or
/// merges them if there already is a node with the same degree in the 
/// root_list.
fn insert_root_list<T>(link: Link<T>, root_list: &mut [Link<T>]) -> Link<T> 
    where
        T: PartialOrd {
    unsafe {
        let cur_spot = (*link).degree as usize;
        if root_list[cur_spot].is_null() {
            root_list[cur_spot] = link;
            ptr::null_mut()
        } else {
            let (min, max) = if (*link).val < (*root_list[cur_spot]).val { 
                (link, root_list[cur_spot])
            } else { 
                (root_list[cur_spot], link)
            };

            (*min).children.push(max);
            (*min).degree += 1;
            root_list[cur_spot] = ptr::null_mut();

            min
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::FibHeap;

    #[test]
    fn insert_none() {
        let mut feap = FibHeap::<i32>::new();
        assert_eq!(feap.get_min(), None);
        assert_eq!(feap.extract_min(), None);
    }

    #[test]
    fn insert_one() {
        let mut feap = FibHeap::new();
        feap.insert(10);
        assert_eq!(feap.get_min(), Some(&10));
        assert_eq!(feap.extract_min(), Some(10));
        assert_eq!(feap.extract_min(), None);
    }

    #[test]
    fn insert_min_first() {
        let mut feap = FibHeap::new();
        feap.insert(10);
        feap.insert(11);
        assert_eq!(feap.get_min(), Some(&10));
    }

    #[test]
    fn insert_min_last() {
        let mut feap = FibHeap::new();
        feap.insert(11);
        feap.insert(10);
        assert_eq!(feap.get_min(), Some(&10));
    }


    #[test]
    fn insert_min() {
        let mut feap = FibHeap::new();
        feap.insert(11);
        feap.insert(10);
        assert_eq!(feap.get_min(), Some(&10));
        feap.insert(4);
        assert_eq!(feap.get_min(), Some(&4));
    }

    #[test]
    fn get_min() {
        let mut feap = FibHeap::new();
        feap.insert(11);
        feap.insert(10);
        feap.insert(4);
        assert_eq!(feap.get_min(), Some(&4));
        assert_eq!(feap.extract_min(), Some(4));
        assert_eq!(feap.get_min(), Some(&10));
        assert_eq!(feap.extract_min(), Some(10));
        assert_eq!(feap.get_min(), Some(&11));
        assert_eq!(feap.extract_min(), Some(11));
        assert_eq!(feap.get_min(), None);
        assert_eq!(feap.extract_min(), None);
    }

    #[test]
    fn cleanup() {
        let mut feap = FibHeap::new();
        feap.insert(0);
        feap.insert(1);
        feap.insert(2);
        feap.insert(3);
        feap.insert(4);
        feap.insert(5);
        assert_eq!(feap.extract_min(), Some(0));
        assert_eq!(feap.head_list.len(), 2);
    }

    #[test]
    fn clear() {
        let mut feap = FibHeap::new();
        feap.insert(0);
        feap.insert(1);
        feap.insert(2);
        feap.insert(3);
        feap.insert(4);
        feap.clear();
        assert_eq!(feap.get_min(), None);
        assert_eq!(feap.head_list.len(), 0);
    }
}