# Bytepair Encoding

## Introduction

Simple Bytepair encdoing implementation in Rust.

## Running Unit Tests

```bash
cargo test
```

## Usage Example

You can use the following code to run the bytepair encoding and decoding:

```rust

fn run_calculate_merges() -> LinkedHashMap<(u16, u16), u16> {
    let input = read_complex_file();
    let ids = convert_to_bytes(&input);
    let merges = calculate_merges_default(ids, 276);
    merges
}

fn main() {
    let merges = run_calculate_merges();
    let vocab = generate_vocab(merges.clone());
    let complex_str = read_complex_file();
    let encoded = encode(complex_str.as_str(), merges.clone());
    let res = decode(encoded.clone(), vocab);
    println!("Decoded: {:?}", res);
    assert_eq!(res, complex_str);
}
```