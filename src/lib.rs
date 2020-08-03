mod point;
use point::Point;

const DEFAULT_SEED: i64 = 0;
const PSIZE: i64 = 2048;

const STRETCH: f64 = -0.211_324_865_405_187; // (1 / sqrt(2 + 1) - 1) / 2
const SQUISH: f64 = 0.366_025_403_784_439; // (sqrt(2 + 1) - 1) / 2

const NORMALIZING_SCALAR: f64 = 47.0;

const STRETCH_POINT: Point<f64> = Point::new(STRETCH, STRETCH);
const SQUISH_POINT: Point<f64> = Point::new(SQUISH, SQUISH);

const GRAD_TABLE: [Point<f64>; 8] = [
    Point::new(5.0, 2.0),
    Point::new(2.0, 5.0),
    Point::new(-5.0, 2.0),
    Point::new(-2.0, 5.0),
    Point::new(5.0, -2.0),
    Point::new(2.0, -5.0),
    Point::new(-5.0, -2.0),
    Point::new(-2.0, -5.0),
];

pub struct OpenSimplexNoise {
    perm: [i64; PSIZE as usize],
}

impl OpenSimplexNoise {
    pub fn new(custom_seed: Option<i64>) -> Self {
        let seed = match custom_seed {
            Some(value) => value,
            None => DEFAULT_SEED,
        };

        let mut perm: [i64; PSIZE as usize] = [0; PSIZE as usize];

        let mut source: [i64; PSIZE as usize] = [0; PSIZE as usize];
        for i in 0..PSIZE {
            source[i as usize] = i;
        }

        for i in (0..PSIZE).rev() {
            let seed: i128 = (seed as i128 * 6_364_136_223_846_793_005) + 1_442_695_040_888_963_407;
            let mut r = ((seed + 31) % (i as i128 + 1)) as i64;
            if r < 0 {
                r += i as i64 + 1;
            }
            perm[i as usize] = source[r as usize];
            source[r as usize] = source[i as usize];
        }

        Self { perm }
    }

    pub fn eval_2d(&self, x: f64, y: f64) -> f64 {
        let input = Point::new(x, y);
        let stretch: Point<f64> = input + (STRETCH_POINT * input.sum());
        let grid = stretch.map(fast_floor).map(to_f64);

        let squashed: Point<f64> = grid + (SQUISH_POINT * grid.sum());
        let ins = stretch - grid;
        let origin = input - squashed;

        self.eval(grid, origin, ins)
    }

    fn eval(&self, grid: Point<f64>, origin: Point<f64>, ins: Point<f64>) -> f64 {
        let mut value = 0.0;
        value += self.contribute(1.0, 0.0, origin, grid);
        value += self.contribute(0.0, 1.0, origin, grid);

        let in_sum = ins.sum();
        if in_sum <= 1.0 {
            // Inside the triangle (2-Simplex) at (0, 0)
            let zins = 1.0 - in_sum;
            if zins > ins.x || zins > ins.y {
                // (0, 0) is one of the closest two triangular vertices
                if ins.x > ins.y {
                    value += self.contribute(1.0, -1.0, origin, grid);
                } else {
                    value += self.contribute(-1.0, 1.0, origin, grid);
                }
            } else {
                // (1, 0) and (0, 1) are the closest two vertices.
                value += self.contribute(1.0, 1.0, origin, grid);
            }

            value += self.contribute(0.0, 0.0, origin, grid);
        } else {
            // Inside the triangle (2-Simplex) at (1, 1)
            let zins = 2.0 - in_sum;
            if zins < ins.x || zins < ins.y {
                // (0, 0) is one of the closest two triangular vertices
                if ins.x > ins.y {
                    value += self.contribute(2.0, 0.0, origin, grid);
                } else {
                    value += self.contribute(0.0, 2.0, origin, grid);
                }
            } else {
                // (1, 0) and (0, 1) are the closest two vertices.
                value += self.contribute(0.0, 0.0, origin, grid);
            }

            value += self.contribute(1.0, 1.0, origin, grid);
        }

        value / NORMALIZING_SCALAR
    }

    fn contribute(&self, dx: f64, dy: f64, origin: Point<f64>, grid: Point<f64>) -> f64 {
        let delta = Point::new(dx, dy);
        let shifted: Point<f64> = origin - delta - SQUISH_POINT * delta.sum();
        let attn: f64 = get_attn(shifted);
        if attn > 0.0 {
            let attn2 = attn * attn;
            return attn2 * attn2 * self.extrapolate(grid + delta, shifted);
        }

        0.0
    }

    fn extrapolate(&self, grid: Point<f64>, delta: Point<f64>) -> f64 {
        let index0 = (self.perm[(grid.x as i64 & 0xFF) as usize] + grid.y as i64) & 0xFF;
        let index1 = (self.perm[index0 as usize] & 0x0E) >> 1;
        let point = GRAD_TABLE[index1 as usize];
        point.x * delta.x + point.y * delta.y
    }
}

fn get_attn(p: Point<f64>) -> f64 {
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
