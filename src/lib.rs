mod open_simplex_noise_2d;
mod open_simplex_noise_3d;
mod utils;
mod vector;

use open_simplex_noise_2d::OpenSimplexNoise2D;
use open_simplex_noise_3d::OpenSimplexNoise3D;
use vector::{vec2::Vec2, vec3::Vec3};

pub const PSIZE: i64 = 2048;
const DEFAULT_SEED: i64 = 0;

type PermTable = [i64; PSIZE as usize];

pub struct OpenSimplexNoise {
    perm: PermTable,
}

impl OpenSimplexNoise {
    pub fn new(custom_seed: Option<i64>) -> Self {
        let seed = match custom_seed {
            Some(value) => value,
            None => DEFAULT_SEED,
        };

        Self {
            perm: generate_perm_array(seed),
        }
    }

    pub fn eval_2d(&self, x: f64, y: f64) -> f64 {
        OpenSimplexNoise2D::eval(Vec2::new(x, y), &self.perm)
    }

    pub fn eval_3d(&self, x: f64, y: f64, z: f64) -> f64 {
        OpenSimplexNoise3D::eval(Vec3::new(x, y, z), &self.perm)
    }
}

pub trait NoiseEvaluator<T: vector::VecType<f64>> {
    const STRETCH_POINT: T;
    const SQUISH_POINT: T;

    fn eval(point: T, perm: &PermTable) -> f64;
    fn extrapolate(grid: T, delta: T, perm: &PermTable) -> f64;
}

fn generate_perm_array(seed: i64) -> PermTable {
    let mut perm: PermTable = [0; PSIZE as usize];

    let mut source: Vec<i64> = (0..PSIZE).map(|x| x).collect();

    for i in (0..PSIZE).rev() {
        let seed: i128 = (seed as i128 * 6_364_136_223_846_793_005) + 1_442_695_040_888_963_407;
        let mut r = ((seed + 31) % (i as i128 + 1)) as i64;
        if r < 0 {
            r += i + 1;
        }
        perm[i as usize] = source[r as usize];
        source[r as usize] = source[i as usize];
    }

    perm
}
