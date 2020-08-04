# OpenSimplex noise in rust [![Crates.io][cr-badge]][cr]
[cr-badge]: https://img.shields.io/crates/v/opensimplex_noise_rs.svg
[cr]: https://crates.io/crates/opensimplex_noise_rs
#### OpenSimplex noise algorithm implementation in Rust

Currently supports only 2d noise.

### Examples:
<img src="https://i.imgur.com/9DCGzJh.png">

### Usage:
```rust
let noise_generator = OpenSimplexNoise::new(Some(883_279_212_983_182_319)); // if not provided, default seed is equal to 0
let scale = 0.044;
let value = noise_generator.eval_2d(x * scale, y * scale); // generates value in range (-1, 1)
```
### Instalation
###### Just add this line to Cargo.toml file in your Rust project
```toml
[dependencies]
opensimplex_noise_rs = "0.1.0"
```
