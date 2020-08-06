mod constants;
mod open_simplex_noise_2d;
mod open_simplex_noise_3d;
mod utils;
mod vector;

use constants::PSIZE;
use open_simplex_noise_2d::OpenSimplexNoise2D;
use open_simplex_noise_3d::OpenSimplexNoise3D;

const DEFAULT_SEED: i64 = 0;

pub struct OpenSimplexNoise {
    perm: [i64; PSIZE as usize],
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
        OpenSimplexNoise2D::eval_2d(x, y, &self.perm)
    }

    pub fn eval_3d(&self, x: f64, y: f64, z: f64) -> f64 {
        OpenSimplexNoise3D::eval_3d(x, y, z, &self.perm)
    }
}

pub trait NoiseEvaluator<T: vector::VecType<f64>> {
    const STRETCH_POINT: T;
    const SQUISH_POINT: T;

    fn extrapolate(grid: T, delta: T, perm: &[i64; PSIZE as usize]) -> f64;
}

fn generate_perm_array(seed: i64) -> [i64; PSIZE as usize] {
    let mut perm: [i64; PSIZE as usize] = [0; PSIZE as usize];

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
