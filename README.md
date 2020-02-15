# TLE Parser

TLE (Two-line elements) parser.

## Example

```rust
extern crate tle_parser;

use tle_parser::parse;

fn main() {
    let raw_tle = "ISS (ZARYA)
1 25544U 98067A   20045.18587073  .00000950  00000-0  25302-4 0  9990
2 25544  51.6443 242.0161 0004885 264.6060 207.3845 15.49165514212791";

    let tle = parse(raw_tle);

    match tle {
        Ok(t) => println!("{:?}", t),
        Err(_) => println!("Error Parsing TLE"),
    }
}
```

You can run this example with the following command:


```
cargo run --example parse_iss_tle
```
