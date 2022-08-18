use feap::FibHeap;

#[test]
fn simple() {
    let mut feap = FibHeap::<i32>::new();
    assert_eq!(feap.get_min(), None);
    assert_eq!(feap.extract_min(), None);
}

#[test]
fn insert_one() {
    let mut feap = FibHeap::new();
    feap.insert(10);
    assert_eq!(feap.get_min(), Some(&10));
}

#[test]
fn insert_multiple() {
    let mut feap = FibHeap::new();
    feap.insert(10);
    feap.insert(11);
    assert_eq!(feap.get_min(), Some(&10));
}

