use super::constants::PSIZE;
use super::vector::vec2::Vec2;

const STRETCH: f64 = -0.211_324_865_405_187; // (1 / sqrt(2 + 1) - 1) / 2
const SQUISH: f64 = 0.366_025_403_784_439; // (sqrt(2 + 1) - 1) / 2

const NORMALIZING_SCALAR: f64 = 47.0;

const STRETCH_POINT: Vec2<f64> = Vec2::new(STRETCH, STRETCH);
const SQUISH_POINT: Vec2<f64> = Vec2::new(SQUISH, SQUISH);

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
impl OpenSimplexNoise2D {
    pub fn eval_2d(x: f64, y: f64, perm: &[i64; PSIZE as usize]) -> f64 {
        let input = Vec2::new(x, y);
        let stretch: Vec2<f64> = input + (STRETCH_POINT * input.sum());
        let grid = stretch.map(fast_floor).map(to_f64);

        let squashed: Vec2<f64> = grid + (SQUISH_POINT * grid.sum());
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
        let mut value = 0.0;
        value += OpenSimplexNoise2D::contribute(1.0, 0.0, origin, grid, perm);
        value += OpenSimplexNoise2D::contribute(0.0, 1.0, origin, grid, perm);

        let in_sum = ins.sum();
        if in_sum <= 1.0 {
            // Inside the triangle (2-Simplex) at (0, 0)
            let zins = 1.0 - in_sum;
            if zins > ins.x || zins > ins.y {
                // (0, 0) is one of the closest two triangular vertices
                if ins.x > ins.y {
                    value += OpenSimplexNoise2D::contribute(1.0, -1.0, origin, grid, perm);
                } else {
                    value += OpenSimplexNoise2D::contribute(-1.0, 1.0, origin, grid, perm);
                }
            } else {
                // (1, 0) and (0, 1) are the closest two vertices.
                value += OpenSimplexNoise2D::contribute(1.0, 1.0, origin, grid, perm);
            }

            value += OpenSimplexNoise2D::contribute(0.0, 0.0, origin, grid, perm);
        } else {
            // Inside the triangle (2-Simplex) at (1, 1)
            let zins = 2.0 - in_sum;
            if zins < ins.x || zins < ins.y {
                // (0, 0) is one of the closest two triangular vertices
                if ins.x > ins.y {
                    value += OpenSimplexNoise2D::contribute(2.0, 0.0, origin, grid, perm);
                } else {
                    value += OpenSimplexNoise2D::contribute(0.0, 2.0, origin, grid, perm);
                }
            } else {
                // (1, 0) and (0, 1) are the closest two vertices.
                value += OpenSimplexNoise2D::contribute(0.0, 0.0, origin, grid, perm);
            }

            value += OpenSimplexNoise2D::contribute(1.0, 1.0, origin, grid, perm);
        }

        value / NORMALIZING_SCALAR
    }

    fn contribute(
        dx: f64,
        dy: f64,
        origin: Vec2<f64>,
        grid: Vec2<f64>,
        perm: &[i64; PSIZE as usize],
    ) -> f64 {
        let delta = Vec2::new(dx, dy);
        let shifted: Vec2<f64> = origin - delta - SQUISH_POINT * delta.sum();
        let attn: f64 = get_attn(shifted);
        if attn > 0.0 {
            let attn2 = attn * attn;
            return attn2 * attn2 * OpenSimplexNoise2D::extrapolate(grid + delta, shifted, perm);
        }

        0.0
    }

    fn extrapolate(grid: Vec2<f64>, delta: Vec2<f64>, perm: &[i64; PSIZE as usize]) -> f64 {
        let index0 = (perm[(grid.x as i64 & 0xFF) as usize] + grid.y as i64) & 0xFF;
        let index1 = (perm[index0 as usize] & 0x0E) >> 1;
        let point = GRAD_TABLE_2D[index1 as usize];
        point.x * delta.x + point.y * delta.y
    }
}

fn get_attn(p: Vec2<f64>) -> f64 {
    2.0 - (p.x * p.x) - (p.y * p.y)
}

fn fast_floor(x: f64) -> i64 {
    let xi = x as i64;
    if x < xi as f64 {
        return xi - 1;
    }
    xi
}

fn to_f64(x: i64) -> f64 {
    x as f64
}
