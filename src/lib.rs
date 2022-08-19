use core::{ptr, cmp::Ordering};

const MAX_DEGREE: usize = 0x20;

type Link<T> = *mut Node<T>;

#[derive(Debug, Clone)]
struct Node<T> {
    parent:   Link<T>,
    children: Vec<Link<T>>,
    degree:   u8,
    marked:   bool,

    val: T
}

impl<T: PartialEq + Eq + PartialOrd + Ord> Node<T> {
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

pub struct FibHeap<T: PartialEq + Eq + PartialOrd + Ord + Clone> {
    min: Link<T>,
    head_list: Vec<Link<T>>
}

impl<T: PartialEq + Eq + PartialOrd + Ord + Clone> Drop for FibHeap<T> {
    fn drop(&mut self) {
        while self.extract_min().is_some() { }
    }
}

impl<T: PartialEq + Eq + PartialOrd + Ord + Clone> Default for FibHeap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: PartialEq + Eq + PartialOrd + Ord + Clone> FibHeap<T> {
    pub fn new() -> Self {
        Self {
            min: ptr::null_mut(),
            head_list: Vec::new()
        }
    }

    pub fn clear(&mut self) {
        while self.extract_min().is_some() { }
    }

    pub fn get_min(&self) -> Option<&T> {
        unsafe { self.min.as_ref().map(|m| &m.val) }
    }

    pub fn insert(&mut self, val: T) {
        let new = Box::into_raw(Box::new(Node::new(val)));
        self.insert_node(new);
    }

    fn insert_node(&mut self, new: Link<T>) {
        unsafe {
            if self.min.is_null() || (*new).val < (*self.min).val {
                self.min = new;
            }
            self.head_list.push(new);
        }
    }

    pub fn extract_min(&mut self) -> Option<T> {
        unsafe {
            if self.min.is_null() {
                return None;
            }

            // Remove all children from min
            for &c in &(*self.min).children {
                self.head_list.push(c);
                (*c).parent = core::ptr::null_mut();
            }

            // Merge trees
            let mut root_list: Vec<Option<Link<T>>> = vec![None; MAX_DEGREE];
            for &c in &self.head_list {
                if c != self.min {
                    let mut tmp = insert_root_list(c, &mut root_list);
                    while tmp.is_some() {
                        tmp = insert_root_list(tmp.unwrap(), &mut root_list);
                    }
                }
            }

            // Update head_list
            self.head_list.clear();
            let ret = Box::from_raw(self.min);
            self.min = ptr::null_mut();

            for c in &root_list {
                if c.is_some() {
                    self.insert_node(c.unwrap());
                }
            }

            Some(ret.val)
        }
    }

    fn find_elem(&self, cur_node: Link<T>, val: &T) -> Option<Link<T>> {
        unsafe {
            for &c in &(*cur_node).children {
                match (*c).val.cmp(val) {
                    Ordering::Equal => return Some(c),
                    Ordering::Less => return self.find_elem(c, val),
                    _ => {}
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

fn insert_root_list<T>(link: Link<T>, root_list: &mut [Option<Link<T>>]) -> Option<Link<T>> 
    where
        T: PartialEq + Eq + PartialOrd + Ord + Clone {
    unsafe {
        let cur_spot = (*link).degree as usize;
        if root_list[cur_spot].is_none() {
            root_list[cur_spot] = Some(link);
            None
        } else {
            let v1 = (*link).val.clone();
            let v2 = 
                (*root_list[cur_spot].unwrap())
                .val
                .clone();
            let (min, max) = if v1 < v2 { 
                (link, root_list[cur_spot].unwrap())
            } else { 
                (root_list[cur_spot].unwrap(), link)
            };
    
            (*min).children.push(max);
            (*min).degree += 1;
            root_list[cur_spot] = None;
            Some(min)
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
        let mut feap = FibHeap::<i32>::new();
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