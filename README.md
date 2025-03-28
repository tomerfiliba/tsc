# tsc

A very simple library that provides rdtsc and the TSC frequency on x86-64 and aarch64.

## Usage
```rust
use tsc::TSC;

let tsc = TSC::new()?;
let t0 = tsc.now_f64();
do_something();
let t1 = tsc.now_f64();
println!("{}", t1 - t0);
```
