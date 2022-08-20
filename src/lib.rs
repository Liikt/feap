use core::ptr;
use introspection::*;
use std::collections::BTreeSet;

const MAX_DEGREE: usize = 0x100;
const CONSOLIDATION_THREASHHOLD: usize = 100;

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

pub struct FibHeap<T: PartialOrd> {
    min: Link<T>,
    head_list: Vec<Link<T>>,
    max_len: usize,
    node_set: BTreeSet<Link<T>>,

    times:    introspection::Timer,
}

impl<T: PartialOrd> Drop for FibHeap<T> {
    fn drop(&mut self) {
        #[cfg(feature = "introspection")]
        println!("{:?}", self.times);

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
            max_len: 0,
            node_set: BTreeSet::new(),

            times: Timer::new("Fibonacci Heap"),
        }
    }

    pub fn clear(&mut self) {
        for &n in self.node_set.iter() {
            unsafe { Box::from_raw(n) };
        }
        self.head_list.clear();
        self.node_set.clear();
        self.min = ptr::null_mut();
    }

    pub fn get_min(&self) -> Option<&T> {
        unsafe { self.min.as_ref().map(|m| &m.val) }
    }

    pub fn insert(&mut self, val: T) {
        let new = Box::into_raw(Box::new(Node::new(val)));
        self.node_set.insert(new);
        self.insert_node(new);
        if self.head_list.len() > CONSOLIDATION_THREASHHOLD {
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

    fn consolidate(&mut self, consider_min: bool) {
        unsafe {
            if self.min.is_null() {
                return;
            }

            if !consider_min {
                self.node_set.remove(&self.min);
            }

            // Remove all children from min
            start_timer!(self.times, TimerHook::RemoveChildHook);
            for &c in &(*self.min).children {
                self.head_list.push(c);
                (*c).parent = core::ptr::null_mut();
            }
            mark_timer!(self.times, TimerHook::RemoveChildHook);

            // Merge trees
            start_timer!(self.times, TimerHook::MergingHook);
            let mut root_list: Vec<Link<T>> = vec![ptr::null_mut(); MAX_DEGREE];
            for &c in &self.head_list {
                self.max_len = std::cmp::max(self.head_list.len(), self.max_len);
                start_timer!(self.times, TimerHook::InnerMergingLoop);
                if consider_min || c != self.min {
                    let mut tmp = insert_root_list(c, &mut root_list, &mut self.times);
                    while tmp.is_some() {
                        tmp = insert_root_list(tmp.unwrap(), &mut root_list, &mut self.times);
                    }
                }
                mark_timer!(self.times, TimerHook::InnerMergingLoop);
            }
            mark_timer!(self.times, TimerHook::MergingHook);

            start_timer!(self.times, TimerHook::UpdatingHook);
            // Update head_list
            self.min = ptr::null_mut();
            self.head_list.clear();

            for &c in &root_list {
                if !c.is_null() {
                    self.insert_node(c);
                }
            }
            mark_timer!(self.times, TimerHook::UpdatingHook);
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

fn insert_root_list<T>(link: Link<T>, root_list: &mut [Link<T>], _timer: &mut Timer) -> Option<Link<T>> 
    where
        T: PartialOrd {
    unsafe {
        let cur_spot = (*link).degree as usize;
        if root_list[cur_spot].is_null() {
            start_timer!(_timer, TimerHook::FastRootListInsert);
            root_list[cur_spot] = link;
            mark_timer!(_timer, TimerHook::FastRootListInsert);
            None
        } else {
            start_timer!(_timer, TimerHook::SlowRootListInsert);
            let (min, max) = if (*link).val < (*root_list[cur_spot]).val { 
                (link, root_list[cur_spot])
            } else { 
                (root_list[cur_spot], link)
            };
            
            (*min).children.push(max);
            (*min).degree += 1;
            root_list[cur_spot] = ptr::null_mut();
            mark_timer!(timer, TimerHook::SlowRootListInsert);
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