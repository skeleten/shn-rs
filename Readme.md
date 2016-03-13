[Online Documentation][docs]

# shn-rs

A simple api to read and write `shn`-Files as found in certain online Games.

It is focused to read and write the data, you should parse it into an internal
Format which is more ergonomical to handle in you program.

## Usage

An example usage, to simply read a into an `ShnFile` object looks like this:

```rust
extern crate shn;
extern crate encoding;

fn main() {
    let file = match std::fs::File::open("path/to/file.shn") {
        Ok(file) => file,
        Err(_) => panic!();
    };
    
    let shn_file = match shn::read_from(file, encoding::all::ASCII) {
        Ok(file) => file,
        Err(_) => panic!();
    }
}
```

[docs]: https://skeleten.github.io/shn-rs/shn
