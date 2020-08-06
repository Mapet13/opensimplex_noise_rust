use super::constants::PSIZE;
use super::utils;
use super::vector::{vec2::Vec2, VecTrait};

use super::NoiseEvaluator;

const STRETCH: f64 = -0.211_324_865_405_187; // (1 / sqrt(2 + 1) - 1) / 2
const SQUISH: f64 = 0.366_025_403_784_439; // (sqrt(2 + 1) - 1) / 2

const NORMALIZING_SCALAR: f64 = 47.0;

const GRAD_TABLE_2D: [Vec2<f64>; 8] = [
    Vec2::new(5.0, 2.0),
    Vec2::new(2.0, 5.0),
    Vec2::new(-5.0, 2.0),
    Vec2::new(-2.0, 5.0),
    Vec2::new(5.0, -2.0),
    Vec2::new(2.0, -5.0),
    Vec2::new(-5.0, -2.0),
    Vec2::new(-2.0, -5.0),
];

pub struct OpenSimplexNoise2D {}

impl NoiseEvaluator<Vec2<f64>> for OpenSimplexNoise2D {
    const STRETCH_POINT: Vec2<f64> = Vec2::new(STRETCH, STRETCH);
    const SQUISH_POINT: Vec2<f64> = Vec2::new(SQUISH, SQUISH);

    fn extrapolate(grid: Vec2<f64>, delta: Vec2<f64>, perm: &[i64; PSIZE as usize]) -> f64 {
        let index0 = (perm[(grid.x as i64 & 0xFF) as usize] + grid.y as i64) & 0xFF;
        let index1 = (perm[index0 as usize] & 0x0E) >> 1;
        let point = GRAD_TABLE_2D[index1 as usize];
        point.x * delta.x + point.y * delta.y
    }
}

impl OpenSimplexNoise2D {
    pub fn eval_2d(x: f64, y: f64, perm: &[i64; PSIZE as usize]) -> f64 {
        let input = Vec2::new(x, y);
        let stretch: Vec2<f64> = input + (Self::STRETCH_POINT * input.sum());
        let grid = stretch.map(utils::fast_floor).map(utils::to_f64);

        let squashed: Vec2<f64> = grid + (Self::SQUISH_POINT * grid.sum());
        let ins = stretch - grid;
        let origin = input - squashed;

        OpenSimplexNoise2D::eval(grid, origin, ins, perm)
    }

    fn eval(
        grid: Vec2<f64>,
        origin: Vec2<f64>,
        ins: Vec2<f64>,
        perm: &[i64; PSIZE as usize],
    ) -> f64 {
        let contribute = |x, y| -> f64 { utils::contribute::<OpenSimplexNoise2D, Vec2<f64>>(Vec2::new(x, y), origin, grid, perm) };

        let mut value = 0.0;
        value += contribute(1.0, 0.0);
        value += contribute(0.0, 1.0);

        let in_sum = ins.sum();
        if in_sum <= 1.0 {
            // Inside the triangle (2-Simplex) at (0, 0)
            let zins = 1.0 - in_sum;
            if zins > ins.x || zins > ins.y {
                // (0, 0) is one of the closest two triangular vertices
                if ins.x > ins.y {
                    value += contribute(1.0, -1.0);
                } else {
                    value += contribute(-1.0, 1.0);
                }
            } else {
                // (1, 0) and (0, 1) are the closest two vertices.
                value += contribute(1.0, 1.0);
            }

            value += contribute(0.0, 0.0);
        } else {
            // Inside the triangle (2-Simplex) at (1, 1)
            let zins = 2.0 - in_sum;
            if zins < ins.x || zins < ins.y {
                // (0, 0) is one of the closest two triangular vertices
                if ins.x > ins.y {
                    value += contribute(2.0, 0.0);
                } else {
                    value += contribute(0.0, 2.0);
                }
            } else {
                // (1, 0) and (0, 1) are the closest two vertices.
                value += contribute(0.0, 0.0);
            }

            value += contribute(1.0, 1.0);
        }

        value / NORMALIZING_SCALAR
    }
}