use std::env::args;
use std::process::exit;

use feap::FibHeap;
use rudac::heap::FibonacciHeap;

const NUM_ENTRIES: u16 = 0x1000;
const EXTRACTS: &[u16; 0x18] = &[0x10, 0x3f, 0x69, 0x78, 0x100, 0x420, 0x532, 0x548, 0x5a5, 0x62d, 0x7d9, 0x803, 0x817, 0x860, 0x874, 0x98f, 0x99c, 0xa4d, 0xb90, 0xd1e, 0xd69, 0xe71, 0xed6, 0x1000];

fn feap_bench() {
    let mut insert_times = Vec::new();
    let mut extract_times = Vec::new();
    for _ in 0..0x10 {
        let mut heap = FibHeap::new();
        let mut expected_min = 0;
        for x in 0..=NUM_ENTRIES {
            let start = unsafe { std::arch::x86_64::_rdtsc() };
            heap.insert(x);
            let end = unsafe { std::arch::x86_64::_rdtsc() };
            insert_times.push(end - start);
            if EXTRACTS.binary_search(&x).is_ok() {
                let start = unsafe { std::arch::x86_64::_rdtsc() };
                let min = heap.extract_min();
                let end = unsafe { std::arch::x86_64::_rdtsc() };
                extract_times.push(end - start);
                assert_eq!(min, Some(expected_min));
                expected_min += 1;
            }
        }
    }

    let in_sum: u64 = insert_times.iter().sum();
    let in_tot = insert_times.len();
    let ext_sum: u64 = extract_times.iter().sum();
    let ext_tot = extract_times.len();

    println!("Avg insert: {:.02} <=> Avg extract: {:.02}", 
        (in_sum as f64)/(in_tot as f64), (ext_sum as f64)/(ext_tot as f64),
    );
}

fn rudac_bench() {
    for _ in 0..0x1000 {
        let mut heap = FibonacciHeap::init_min();
        let mut expected_min = 0;
        for x in 0..=NUM_ENTRIES {
            heap.push(x);
            if EXTRACTS.binary_search(&x).is_ok() {
                assert_eq!(heap.pop(), Some(expected_min));
                expected_min += 1;
            }
        }
    }
}

fn main() {
    if args().len() != 2 {
        println!("{:?} <which>", args().next().unwrap());
        exit(1);
    }
    let arg = args().nth(1).unwrap();
    match arg.as_str() {
        "feap" => feap_bench(),
        "rudac" => rudac_bench(),
        _ => panic!("Nope {}", arg)
    }
}