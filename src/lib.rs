use core::ptr;

const MAX_DEGREE: usize = 0x100;
const CONSOLIDATION_THRESHOLD: usize = 100;

type Link<T> = *mut Node<T>;

#[derive(Debug)]
struct Node<T> {
    parent:   Link<T>,
    children: Vec<Link<T>>,
    degree:   u8,
    marked:   bool,

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

#[derive(Clone)]
pub struct FibHeap<T: PartialOrd> {
    min: Link<T>,
    head_list: Vec<Link<T>>,
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
    pub fn new() -> Self {
        Self {
            min: ptr::null_mut(),
            head_list: Vec::new(),
            root_list: vec![ptr::null_mut(); MAX_DEGREE],
        }
    }

    fn _clear(&self, node: Link<T>) {
        unsafe {
            let children = &(*node).children;
            for &c in children {
                self._clear(c);
            }
            drop(Box::from_raw(node));
        }
    }

    pub fn clear(&mut self) {
        for &t in &self.head_list {
            self._clear(t);
        }
        self.head_list.clear();
        self.min = ptr::null_mut();
    }

    pub fn get_min(&self) -> Option<&T> {
        unsafe { self.min.as_ref().map(|m| &m.val) }
    }

    pub fn insert(&mut self, val: T) {
        let new = Box::into_raw(Box::new(Node::new(val)));
        self.insert_node(new);
        if self.head_list.len() > CONSOLIDATION_THRESHOLD {
            self.consolidate(true);
        }
    }

    fn insert_node(&mut self, new: Link<T>) {
        unsafe {
            if self.min.is_null() || (*new).val < (*self.min).val {
                self.min = new;
            }
            self.head_list.push(new);
        }
    }

    fn consolidate(&mut self, insert_mode: bool) {
        unsafe {
            if self.min.is_null() {
                return;
            }

            if !insert_mode {
                // Remove all children from min
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

    fn find_elem(&self, cur_node: Link<T>, val: &T) -> Option<Link<T>> {
        unsafe {
            for &c in &(*cur_node).children {
                if (*c).val.lt(val) {
                    return Some(c)
                } else if (*c).val.eq(val) {
                    return self.find_elem(c, val);
                }
            }
            None
        }
    }

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