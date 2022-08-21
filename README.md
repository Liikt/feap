# Feap

Feap is a fast implementation of a 
[fibonacci heap]((https://en.wikipedia.org/wiki/Fibonacci_heap)). It is the (to
my knowledge) fastest implementation out yet. Inspired was this project by a
youtube video from [SithDev](https://www.youtube.com/watch?v=6JxvKfSV9Ns) that 
got recommended to me.

## Example

```rust
use feap::FibHeap;

fn main() {
    let mut feap = FibHeap::new();

    feap.insert(10);
    feap.insert(4);
    feap.insert(30);

    assert_eq!(feap.extract_min(), Some(4));
    assert_eq!(feap.get_min(), Some(&10));

    feap.decrease_key(30, 7);

    assert_eq!(feap.get_min(), Some(&7));

    feap.clear();

    assert_eq!(feap.get_min(), None);
    assert_eq!(feap.extract_min(), None);
}
```