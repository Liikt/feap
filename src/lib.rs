use std::rc::Rc;
use std::cell::RefCell;

const MAX_DEGREE: usize = 0x20;

#[derive(Debug, Clone)]
struct InnerNode<T> {
    parent:   Option<Node<T>>,
    children: Vec<Node<T>>,
    degree:   u8,

    pub val: T
}

impl<T: PartialEq> PartialEq for InnerNode<T> {
    fn eq(&self, other: &Self) -> bool {
        self.val == other.val
    }
}

impl<T: Eq> Eq for InnerNode<T> {}

impl<T: PartialOrd> PartialOrd for InnerNode<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.val.partial_cmp(&other.val)
    }
}

impl<T: Ord> Ord for InnerNode<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.val.cmp(&other.val)
    }

    fn min(self, other: Self) -> Self
        where
            Self: Sized, {
        if self.val <= other.val {
            self
        } else {
            other
        }
    }
}

impl<T: PartialEq + Eq + PartialOrd + Ord> InnerNode<T> {
    fn new(val: T) -> Self {
        Self {
            parent:   None,
            children: Vec::new(),
            degree:   0,
            val
        }
    }
}

type Node<T> = Rc<RefCell<InnerNode<T>>>; 

pub struct FibHeap<T> {
    min: Option<Node<T>>,
    head_list: Vec<Node<T>>
}

impl<T: PartialEq + Eq + PartialOrd + Ord + Clone> FibHeap<T> {
    pub fn new() -> Self {
        Self {
            min: None,
            head_list: Vec::new()
        }
    }

    pub fn get_min(&self) -> Option<T> {
        if let Some(min) = &self.min {
            return Some(RefCell::borrow(min).val.clone());
        }
        None
    }

    pub fn insert(&mut self, val: T) {
        let new = Rc::new(RefCell::new(InnerNode::new(val)));
        self.insert_node(new);
    }

    fn insert_node(&mut self, new: Node<T>) {
        self.min.as_ref().map_or(Some(new.clone()), |min| {
            if RefCell::borrow(&new).val < RefCell::borrow(min).val {
                return Some(new.clone());
            }
            Some(min.clone())
        });
        self.head_list.push(new);
    }

    pub fn extract_min(&mut self) -> Option<T> {
        if self.min.is_none() {
            return None;
        }

        // Remove all children from min
        for c in &self.min.as_ref().unwrap().borrow().children {
            self.head_list.push(c.clone());
            drop(c);
        }

        // Merge trees
        let mut root_list: Vec<Option<Node<T>>> = vec![None; MAX_DEGREE];
        for c in &self.head_list {
            let mut tmp = insert_root_list(c.clone(), &mut root_list);
            while tmp.is_some() {
                tmp = insert_root_list(tmp.unwrap(), &mut root_list);
            }
        }

        // Update head_list
        self.head_list.clear();
        for c in &root_list {
            if c.is_some() {
                self.insert_node(c.as_ref().unwrap().clone());
            }
        }

        Some(self.min.as_ref().unwrap().borrow().val.clone())
    }

    fn find_elem(&self, cur_node: Node<T>, val: T) -> Option<Node<T>> {
        for c in &RefCell::borrow(&cur_node).children {
            let child_val = RefCell::borrow(&c).val.clone();
            if child_val == val { return Some(c.clone()) }
            else if child_val < val { return self.find_elem(c.clone(), val); }
        }
        None
    }

    pub fn decrease_key(&mut self, old_val: T, new_val: T) {
        let mut cur_node = None;
        for t in &self.head_list {
            cur_node = self.find_elem(t.clone(), old_val.clone());
            if cur_node.is_some() { break; }
        }
        let cur_node = cur_node.as_ref().unwrap();
        RefCell::borrow_mut(&cur_node).val = new_val.clone();
        let parent = RefCell::borrow(&cur_node).parent.clone();
        if parent.is_some() && RefCell::borrow(&parent.as_ref().unwrap()).val >= new_val {
            let parent = parent.as_ref().unwrap();
            
        }
    } 
}

fn insert_root_list<T>(link: Node<T>, root_list: &mut Vec<Option<Node<T>>>) -> Option<Node<T>> 
    where
        T: PartialEq + Eq + PartialOrd + Ord + Clone {
    let cur_spot = RefCell::borrow(&link).degree as usize;
    if root_list[cur_spot].is_none() {
        root_list[cur_spot] = Some(link.clone());
        return None;
    } else {
        let v1 = RefCell::borrow(&link).val.clone();
        let v2 = 
            RefCell::borrow(&root_list[cur_spot].as_ref().unwrap())
            .val
            .clone();
        let (min, max) = if v1 < v2 { 
            (link.clone(), root_list[cur_spot].as_ref().unwrap().clone())
        } else { 
            (root_list[cur_spot].as_ref().unwrap().clone(), link.clone())
        };

        RefCell::borrow_mut(&min).children.push(max);
        RefCell::borrow_mut(&min).degree += 1;
        root_list[cur_spot] = None;
        return Some(min.clone());
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
