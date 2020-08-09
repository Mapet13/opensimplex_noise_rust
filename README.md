# OpenSimplex noise in rust [![Crates.io][cr-badge]][cr]
[cr-badge]: https://img.shields.io/crates/v/opensimplex_noise_rs.svg
[cr]: https://crates.io/crates/opensimplex_noise_rs
[OpenSimplex noise](https://en.wikipedia.org/wiki/OpenSimplex_noise) is a random noise [algorithm by Kurt Spencer](https://uniblock.tumblr.com/post/97868843242/noise), made as a patent-free alternative to Perlin and Simplex noise.

This Rust port currently supports 2D, 3D and 4D noise.

### Examples:
[![example](examples/demo_3d/examples/noise_3d_example.gif)](https://github.com/Mapet13/opensimplex_noise_rust/tree/master/examples/demo_3d)

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
opensimplex_noise_rs = "0.3.0"
```
### Code Examples:
 - [2D Demo](https://github.com/Mapet13/opensimplex_noise_rust/tree/master/examples/demo/)
 - [3D Demo](https://github.com/Mapet13/opensimplex_noise_rust/tree/master/examples/demo_3d)
 - [4D Demo](https://github.com/Mapet13/opensimplex_noise_rust/tree/master/examples/demo_4d)
 - [Island Terrain Generator](https://github.com/Mapet13/terrain-generator-2d)

### License
###### This code is under the same "license" as Kurt's OpenSimplex - the public domain "unlicense."
