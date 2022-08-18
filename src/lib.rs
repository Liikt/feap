use std::{ptr, cmp::Ordering};

const MAX_DEGREE: usize = 0x20;

type Link<T> = *mut Node<T>;

macro_rules! node {
    ($node:expr) => {
        unsafe { Box::from_raw($node) }
    };
}

#[derive(Debug, Clone)]
struct Node<T> {
    parent:   Option<Link<T>>,
    children: Vec<Link<T>>,
    degree:   u8,
    marked:   bool,

    pub val: T
}

impl<T: PartialEq + Eq + PartialOrd + Ord> Node<T> {
    fn new(val: T) -> Self {
        Self {
            parent:   None,
            children: Vec::new(),
            degree:   0,
            marked:   false,
            val
        }
    }
}

#[derive(Debug)]
pub struct FibHeap<T> {
    min: Link<T>,
    head_list: Vec<Link<T>>
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

    pub fn get_min(&self) -> Option<&T> {
        unsafe {self.min.as_ref()}.map(|m| &m.val)
    }

    pub fn insert(&mut self, val: T) {
        let new = Box::into_raw(Box::new(Node::new(val)));
        self.insert_node(new);
    }

    fn insert_node(&mut self, new: Link<T>) {
        if self.min.is_null() || node!(new).val < node!(self.min).val {
            self.min = new;
        }

        self.head_list.push(new);
    }

    fn cleanup(&mut self) {
        // Merge trees
        let mut root_list: Vec<Option<Link<T>>> = vec![None; MAX_DEGREE];
        for &c in &self.head_list {
            let mut tmp = insert_root_list(c, &mut root_list);
            while tmp.is_some() {
                tmp = insert_root_list(tmp.unwrap(), &mut root_list);
            }
        }

        // Update head_list
        self.head_list.clear();
        for c in &root_list {
            if c.is_some() {
                self.insert_node(c.unwrap());
            }
        }
    }

    pub fn extract_min(&mut self) -> Option<T> {
        if self.min.is_null() {
            return None;
        }

        // Remove all children from min
        for c in node!(self.min).children {
            self.head_list.push(c);
            node!(c).parent = None;
        }

        if self.head_list.len() > 8 {
            self.cleanup();
        }

        Some(node!(self.min).val)
    }

    fn find_elem(&self, cur_node: Link<T>, val: &T) -> Option<Link<T>> {
        for &c in &node!(cur_node).children {
            match node!(c).val.cmp(val) {
                Ordering::Equal => return Some(c),
                Ordering::Less => return self.find_elem(c, val),
                _ => {}
            }
        }
        None
    }

    fn cut_out(&mut self, node: Link<T>) {
        node!(node).marked = false;
        if let Some(parent) = node!(node).parent {
            self.insert_node(node);
            let idx = node!(parent).children.iter()
                .position(|&v| node!(v).val == node!(node).val)
                .unwrap();
            node!(parent).children.remove(idx);
            if !node!(parent).marked {
                node!(parent).marked = true;
            } else {
                self.cut_out(parent);
            }
        }
    }


    pub fn decrease_key(&mut self, old_val: T, new_val: T) {
        let mut cur_node = None;
        for &t in &self.head_list {
            cur_node = self.find_elem(t, &old_val);
            if cur_node.is_some() { break; }
        }

        if let Some(cur_node) = cur_node {
            let parent = node!(cur_node).parent;
            if parent.is_some() && node!(parent.unwrap()).val >= new_val {
                self.cut_out(cur_node);
            }
        }

    } 
}

fn insert_root_list<T>(link: Link<T>, root_list: &mut [Option<Link<T>>]) -> Option<Link<T>> 
    where
        T: PartialEq + Eq + PartialOrd + Ord + Clone {
    let cur_spot = node!(link).degree as usize;
    if root_list[cur_spot].is_none() {
        root_list[cur_spot] = Some(link);
        None
    } else {
        let v1 = node!(link).val;
        let v2 = 
            node!(root_list[cur_spot].unwrap())
            .val
            .clone();
        let (min, max) = if v1 < v2 { 
            (link, root_list[cur_spot].unwrap())
        } else { 
            (root_list[cur_spot].unwrap(), link)
        };

        node!(min).children.push(max);
        node!(min).degree += 1;
        root_list[cur_spot] = None;
        Some(min)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
