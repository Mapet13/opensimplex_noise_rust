use super::{
    utils,
    vector::{vec4::Vec4, VecMethods},
    NoiseEvaluator, PermTable,
};
use crate::vector::vec2::Vec2;

const STRETCH: f64 = -0.138_196_601_125_011; // (1 / sqrt(4 + 1) - 1) / 4
const SQUISH: f64 = 0.309_016_994_374_947; // (sqrt(4 + 1) - 1) / 4

const NORMALIZING_SCALAR: f64 = 30.0;

const GRAD_TABLE: [Vec4<f64>; 64] = [
    Vec4::new(3.0, 1.0, 1.0, 1.0),
    Vec4::new(1.0, 3.0, 1.0, 1.0),
    Vec4::new(1.0, 1.0, 3.0, 1.0),
    Vec4::new(1.0, 1.0, 1.0, 3.0),
    Vec4::new(-3.0, 1.0, 1.0, 1.0),
    Vec4::new(-1.0, 3.0, 1.0, 1.0),
    Vec4::new(-1.0, 1.0, 3.0, 1.0),
    Vec4::new(-1.0, 1.0, 1.0, 3.0),
    Vec4::new(3.0, -1.0, 1.0, 1.0),
    Vec4::new(1.0, -3.0, 1.0, 1.0),
    Vec4::new(1.0, -1.0, 3.0, 1.0),
    Vec4::new(1.0, -1.0, 1.0, 3.0),
    Vec4::new(-3.0, -1.0, 1.0, 1.0),
    Vec4::new(-1.0, -3.0, 1.0, 1.0),
    Vec4::new(-1.0, -1.0, 3.0, 1.0),
    Vec4::new(-1.0, -1.0, 1.0, 3.0),
    Vec4::new(3.0, 1.0, -1.0, 1.0),
    Vec4::new(1.0, 3.0, -1.0, 1.0),
    Vec4::new(1.0, 1.0, -3.0, 1.0),
    Vec4::new(1.0, 1.0, -1.0, 3.0),
    Vec4::new(-3.0, 1.0, -1.0, 1.0),
    Vec4::new(-1.0, 3.0, -1.0, 1.0),
    Vec4::new(-1.0, 1.0, -3.0, 1.0),
    Vec4::new(-1.0, 1.0, -1.0, 3.0),
    Vec4::new(3.0, -1.0, -1.0, 1.0),
    Vec4::new(1.0, -3.0, -1.0, 1.0),
    Vec4::new(1.0, -1.0, -3.0, 1.0),
    Vec4::new(1.0, -1.0, -1.0, 3.0),
    Vec4::new(-3.0, -1.0, -1.0, 1.0),
    Vec4::new(-1.0, -3.0, -1.0, 1.0),
    Vec4::new(-1.0, -1.0, -3.0, 1.0),
    Vec4::new(-1.0, -1.0, -1.0, 3.0),
    Vec4::new(3.0, 1.0, 1.0, -1.0),
    Vec4::new(1.0, 3.0, 1.0, -1.0),
    Vec4::new(1.0, 1.0, 3.0, -1.0),
    Vec4::new(1.0, 1.0, 1.0, -3.0),
    Vec4::new(-3.0, 1.0, 1.0, -1.0),
    Vec4::new(-1.0, 3.0, 1.0, -1.0),
    Vec4::new(-1.0, 1.0, 3.0, -1.0),
    Vec4::new(-1.0, 1.0, 1.0, -3.0),
    Vec4::new(3.0, -1.0, 1.0, -1.0),
    Vec4::new(1.0, -3.0, 1.0, -1.0),
    Vec4::new(1.0, -1.0, 3.0, -1.0),
    Vec4::new(1.0, -1.0, 1.0, -3.0),
    Vec4::new(-3.0, -1.0, 1.0, -1.0),
    Vec4::new(-1.0, -3.0, 1.0, -1.0),
    Vec4::new(-1.0, -1.0, 3.0, -1.0),
    Vec4::new(-1.0, -1.0, 1.0, -3.0),
    Vec4::new(3.0, 1.0, -1.0, -1.0),
    Vec4::new(1.0, 3.0, -1.0, -1.0),
    Vec4::new(1.0, 1.0, -3.0, -1.0),
    Vec4::new(1.0, 1.0, -1.0, -3.0),
    Vec4::new(-3.0, 1.0, -1.0, -1.0),
    Vec4::new(-1.0, 3.0, -1.0, -1.0),
    Vec4::new(-1.0, 1.0, -3.0, -1.0),
    Vec4::new(-1.0, 1.0, -1.0, -3.0),
    Vec4::new(3.0, -1.0, -1.0, -1.0),
    Vec4::new(1.0, -3.0, -1.0, -1.0),
    Vec4::new(1.0, -1.0, -3.0, -1.0),
    Vec4::new(1.0, -1.0, -1.0, -3.0),
    Vec4::new(-3.0, -1.0, -1.0, -1.0),
    Vec4::new(-1.0, -3.0, -1.0, -1.0),
    Vec4::new(-1.0, -1.0, -3.0, -1.0),
    Vec4::new(-1.0, -1.0, -1.0, -3.0),
];

pub struct OpenSimplexNoise4D {}

impl NoiseEvaluator<Vec4<f64>> for OpenSimplexNoise4D {
    const STRETCH_POINT: Vec4<f64> = Vec4::new(STRETCH, STRETCH, STRETCH, STRETCH);
    const SQUISH_POINT: Vec4<f64> = Vec4::new(SQUISH, SQUISH, SQUISH, SQUISH);

    fn extrapolate(grid: Vec4<f64>, delta: Vec4<f64>, perm: &PermTable) -> f64 {
        let point = GRAD_TABLE[Self::get_grad_table_index(grid, perm)];

        point.x * delta.x + point.y * delta.y + point.z * delta.z + point.w * delta.w
    }

    fn eval(input: Vec4<f64>, perm: &PermTable) -> f64 {
        let stretch: Vec4<f64> = input + (Self::STRETCH_POINT * input.sum());
        let grid = stretch.map(utils::floor).map(utils::to_f64);

        let squashed: Vec4<f64> = grid + (Self::SQUISH_POINT * grid.sum());
        let ins = stretch - grid;
        let origin = input - squashed;

        Self::get_value(grid, origin, ins, perm)
    }
}

impl OpenSimplexNoise4D {
    fn inside_pentachoron_at_0_0_0_0(
        ins: Vec4<f64>,
        contribute: impl Fn(f64, f64, f64, f64) -> f64,
    ) -> f64 {
        let mut point = Vec2::new(1, 2);
        let mut score = ins;
        if score.x >= score.y && ins.z > score.y {
            score.y = ins.z;
            point.y = 4;
        } else if score.x < score.y && ins.z > score.x {
            score.x = ins.z;
            point.x = 4;
        }
        if score.x >= score.y && ins.w > score.y {
            score.y = ins.w;
            point.y = 8;
        } else if score.x < score.y && ins.w > score.x {
            score.x = ins.w;
            point.x = 8;
        }

        // Now we determine the three lattice points not part of the pentachoron that may contribute.
        // This depends on the closest two pentachoron vertices, including (0, 0, 0, 0)
        let uins = 1.0 - ins.sum();
        let value = if uins > score.x || uins > score.y {
            // (0, 0, 0, 0) is one of the closest two pentachoron vertices.
            // Our other closest vertex is the closest out of a and b.
            let closest = if score.y > score.x { point.y } else { point.x };
            match closest {
                1 => {
                    contribute(1.0, -1.0, 0.0, 0.0)
                        + contribute(1.0, 0.0, -1.0, 0.0)
                        + contribute(1.0, 0.0, 0.0, -1.0)
                }
                2 => {
                    contribute(-1.0, 1.0, 0.0, 0.0)
                        + contribute(0.0, 1.0, -1.0, 0.0)
                        + contribute(0.0, 1.0, 0.0, -1.0)
                }
                4 => {
                    contribute(-1.0, 0.0, 1.0, 0.0)
                        + contribute(0.0, -1.0, 1.0, 0.0)
                        + contribute(0.0, 0.0, 1.0, -1.0)
                }
                _ => {
                    // closest == 8
                    contribute(-1.0, 0.0, 0.0, 1.0)
                        + contribute(0.0, -1.0, 0.0, 1.0)
                        + contribute(0.0, 0.0, -1.0, 1.0)
                }
            }
        } else {
            // (0, 0, 0, 0) is not one of the closest two pentachoron vertices.
            // Our three extra vertices are determined by the closest two.
            let closest = point.x | point.y;
            match closest {
                3 => {
                    contribute(1.0, 1.0, 0.0, 0.0)
                        + contribute(1.0, 1.0, -1.0, 0.0)
                        + contribute(1.0, 1.0, 0.0, -1.0)
                }
                5 => {
                    contribute(1.0, 0.0, 1.0, 0.0)
                        + contribute(1.0, -1.0, 1.0, 0.0)
                        + contribute(1.0, 0.0, 1.0, -1.0)
                }
                6 => {
                    contribute(0.0, 1.0, 1.0, 0.0)
                        + contribute(-1.0, 1.0, 1.0, 0.0)
                        + contribute(0.0, 1.0, 1.0, -1.0)
                }
                9 => {
                    contribute(1.0, 0.0, 0.0, 1.0)
                        + contribute(1.0, -1.0, 0.0, 1.0)
                        + contribute(1.0, 0.0, -1.0, 1.0)
                }
                10 => {
                    contribute(0.0, 1.0, 0.0, 1.0)
                        + contribute(-1.0, 1.0, 0.0, 1.0)
                        + contribute(0.0, 1.0, -1.0, 1.0)
                }
                _ => {
                    // closest == 12
                    contribute(0.0, 0.0, 1.0, 1.0)
                        + contribute(-1.0, 0.0, 1.0, 1.0)
                        + contribute(0.0, -1.0, 1.0, 1.0)
                }
            }
        };

        value
            + contribute(0.0, 0.0, 0.0, 0.0)
            + contribute(1.0, 0.0, 0.0, 0.0)
            + contribute(0.0, 1.0, 0.0, 0.0)
            + contribute(0.0, 0.0, 1.0, 0.0)
            + contribute(0.0, 0.0, 0.0, 1.0)
    }

    fn inside_pentachoron_at_1_1_1_1(
        ins: Vec4<f64>,
        contribute: impl Fn(f64, f64, f64, f64) -> f64,
    ) -> f64 {
        let mut point = Vec2::new(14, 13);
        let mut score = ins;
        if score.x <= score.y && ins.z < score.y {
            score.y = ins.z;
            point.y = 11;
        } else if score.x > score.y && ins.z < score.x {
            score.x = ins.z;
            point.x = 11;
        }
        if score.x <= score.y && ins.w < score.y {
            score.y = ins.w;
            point.y = 7;
        } else if score.x > score.y && ins.w < score.x {
            score.x = ins.w;
            point.x = 7;
        }

        // Now we determine the three lattice points not part of the pentachoron that may contribute.
        // This depends on the closest two pentachoron vertices, including (0, 0, 0, 0)
        let uins = 1.0 - ins.sum();
        let value = if uins < score.x || uins > score.y {
            // (0, 0, 0, 0) is one of the closest two pentachoron vertices.
            // Our other closest vertex is the closest out of a and b.
            let closest = if score.y < score.x { point.y } else { point.x };
            match closest {
                7 => {
                    contribute(2.0, 1.0, 1.0, 0.0)
                        + contribute(1.0, 2.0, 1.0, 0.0)
                        + contribute(1.0, 1.0, 2.0, 0.0)
                }
                11 => {
                    contribute(2.0, 1.0, 0.0, 1.0)
                        + contribute(1.0, 2.0, 0.0, 1.0)
                        + contribute(1.0, 1.0, 0.0, 2.0)
                }
                13 => {
                    contribute(2.0, 0.0, 1.0, 1.0)
                        + contribute(1.0, 0.0, 2.0, 1.0)
                        + contribute(1.0, 0.0, 1.0, 2.0)
                }
                _ => {
                    // closest == 14
                    contribute(0.0, 2.0, 1.0, 1.0)
                        + contribute(0.0, 1.0, 2.0, 1.0)
                        + contribute(0.0, 1.0, 1.0, 2.0)
                }
            }
        } else {
            // (0, 0, 0, 0) is not one of the closest two pentachoron vertices.
            // Our three extra vertices are determined by the closest two.
            let closest = point.x & point.y;
            match closest {
                3 => {
                    contribute(1.0, 1.0, 0.0, 0.0)
                        + contribute(2.0, 1.0, 0.0, 0.0)
                        + contribute(1.0, 2.0, 0.0, 0.0)
                }
                5 => {
                    contribute(1.0, 0.0, 1.0, 0.0)
                        + contribute(2.0, 0.0, 1.0, 0.0)
                        + contribute(1.0, 0.0, 2.0, 0.0)
                }
                6 => {
                    contribute(0.0, 1.0, 1.0, 0.0)
                        + contribute(0.0, 2.0, 1.0, 0.0)
                        + contribute(0.0, 1.0, 2.0, 0.0)
                }
                9 => {
                    contribute(1.0, 0.0, 0.0, 1.0)
                        + contribute(2.0, 0.0, 0.0, 1.0)
                        + contribute(1.0, 0.0, 0.0, 2.0)
                }
                10 => {
                    contribute(0.0, 1.0, 0.0, 1.0)
                        + contribute(0.0, 2.0, 0.0, 1.0)
                        + contribute(0.0, 1.0, 0.0, 2.0)
                }
                _ => {
                    // closest == 12
                    contribute(0.0, 0.0, 1.0, 1.0)
                        + contribute(0.0, 0.0, 2.0, 1.0)
                        + contribute(0.0, 0.0, 1.0, 2.0)
                }
            }
        };

        value
            + contribute(1.0, 1.0, 1.0, 0.0)
            + contribute(1.0, 1.0, 0.0, 1.0)
            + contribute(1.0, 0.0, 1.0, 1.0)
            + contribute(0.0, 1.0, 1.0, 1.0)
            + contribute(1.0, 1.0, 1.0, 1.0)
    }

    fn inside_second_dispentachoron(
        ins: Vec4<f64>,
        contribute: impl Fn(f64, f64, f64, f64) -> f64,
    ) -> f64 {
        let mut value = 0.0;

        let mut score = Vec2::new(0.0, 0.0);
        let mut point = Vec2::new(0, 0);
        let mut is_bigger_side = Vec2::new(true, true);

        // Decide between (0,0,1,1) and (1,1,0,0)
        if ins.x + ins.y < ins.z + ins.w {
            score.x = ins.x + ins.y;
            point.x = 12;
        } else {
            score.x = ins.z + ins.w;
            point.x = 3;
        }

        // Decide between (0,1,0,1) and (1,0,1,0)
        if ins.x + ins.z < ins.y + ins.w {
            score.y = ins.x + ins.z;
            point.y = 10;
        } else {
            score.y = ins.y + ins.w;
            point.y = 5;
        }

        // Closer between (0,1,1,0) and (1,0,0,1) will replace the further of a and b,
        // if closer.
        if ins.x + ins.w < ins.y + ins.z {
            let score_value = ins.x + ins.w;
            if score.x <= score.y && score_value < score.y {
                score.y = score_value;
                point.y = 6;
            } else if score.x > score.y && score_value < score.x {
                score.x = score_value;
                point.x = 6;
            }
        } else {
            let score_value = ins.y + ins.z;
            if score.x <= score.y && score_value < score.y {
                score.y = score_value;
                point.y = 9;
            } else if score.x > score.y && score_value < score.x {
                score.x = score_value;
                point.x = 9;
            }
        }

        // Decide if (0, 1, 1, 1) is closer.
        let p1 = 3.0 - ins.sum() + ins.x;
        if score.x <= score.y && p1 < score.y {
            score.y = p1;
            point.y = 14;
            is_bigger_side.y = false;
        } else if score.x > score.y && p1 < score.x {
            score.x = p1;
            point.x = 14;
            is_bigger_side.x = false;
        }

        // Decide if (1, 0, 1, 1) is closer.
        let p2 = 3.0 - ins.sum() + ins.y;
        if score.x <= score.y && p2 < score.y {
            score.y = p2;
            point.y = 13;
            is_bigger_side.y = false;
        } else if score.x > score.y && p2 < score.x {
            score.x = p2;
            point.x = 13;
            is_bigger_side.x = false;
        }

        // Decide if (1, 1, 0, 1) is closer.
        let p3 = 3.0 - ins.sum() + ins.z;
        if score.x <= score.y && p3 < score.y {
            score.y = p3;
            point.y = 11;
            is_bigger_side.y = false;
        } else if score.x > score.y && p3 < score.x {
            score.x = p3;
            point.x = 11;
            is_bigger_side.x = false;
        }

        // Decide if (1, 1, 1, 0) is closer.
        let p4 = 3.0 - ins.sum() + ins.w;
        if score.x <= score.y && p4 < score.y {
            point.y = 7;
            is_bigger_side.y = false;
        } else if score.x > score.y && p4 < score.x {
            point.x = 7;
            is_bigger_side.x = false;
        }

        // Where each of the two closest points are determines how the extra three
        // vertices are calculated.
        if is_bigger_side.x == is_bigger_side.y {
            if is_bigger_side.x {
                // Both closest points on the bigger side

                // Two contributions are permutations of (0, 0, 0, 1) and (0, 0, 0, 2) based on c1
                let c1 = point.x & point.y;
                value += match c1 {
                    1 => contribute(1.0, 0.0, 0.0, 0.0) + contribute(2.0, 0.0, 0.0, 0.0),
                    2 => contribute(0.0, 1.0, 0.0, 0.0) + contribute(0.0, 2.0, 0.0, 0.0),
                    4 => contribute(0.0, 0.0, 1.0, 0.0) + contribute(0.0, 0.0, 2.0, 0.0),
                    _ => contribute(0.0, 0.0, 0.0, 1.0) + contribute(0.0, 0.0, 0.0, 2.0), // c2 == 8
                };

                // One contribution is a permutation of (1, 1, 1, -1) based on c2
                let c2 = point.x | point.y;

                value += match c2 {
                    x if (x & 1) == 0 => contribute(-1.0, 1.0, 1.0, 1.0),
                    x if (x & 2) == 0 => contribute(1.0, -1.0, 1.0, 1.0),
                    x if (x & 4) == 0 => contribute(1.0, 1.0, -1.0, 1.0),
                    _ => contribute(1.0, 1.0, 1.0, -1.0), // (c2 & 8) == 0
                };
            } else {
                // Both closest points on the smaller side
                // One of the two extra points is (1, 1, 1, 1)
                value += contribute(1.0, 1.0, 1.0, 1.0);

                // Other two points are based on the shared axes.
                let closest = point.x & point.y;
                value += match closest {
                    3 => contribute(2.0, 1.0, 0.0, 0.0) + contribute(1.0, 2.0, 0.0, 0.0),
                    5 => contribute(2.0, 0.0, 1.0, 0.0) + contribute(1.0, 0.0, 2.0, 0.0),
                    6 => contribute(0.0, 2.0, 1.0, 0.0) + contribute(0.0, 1.0, 2.0, 0.0),
                    9 => contribute(2.0, 0.0, 0.0, 1.0) + contribute(1.0, 0.0, 0.0, 2.0),
                    10 => contribute(0.0, 2.0, 0.0, 1.0) + contribute(0.0, 1.0, 0.0, 2.0),
                    _ => contribute(0.0, 0.0, 2.0, 1.0) + contribute(0.0, 0.0, 1.0, 2.0), // closest == 12
                };
            }
        } else {
            // One point on each "side"
            let (c1, c2) = if is_bigger_side.x {
                (point.x, point.y)
            } else {
                (point.y, point.x)
            };

            // Two contributions are the bigger-sided point with each 1 replaced with 2.
            value += match c1 {
                3 => contribute(2.0, 1.0, 0.0, 0.0) + contribute(1.0, 2.0, 0.0, 0.0),
                5 => contribute(2.0, 0.0, 1.0, 0.0) + contribute(1.0, 0.0, 2.0, 0.0),
                6 => contribute(0.0, 2.0, 1.0, 0.0) + contribute(0.0, 1.0, 2.0, 0.0),
                9 => contribute(2.0, 0.0, 0.0, 1.0) + contribute(1.0, 0.0, 0.0, 2.0),
                10 => contribute(0.0, 2.0, 0.0, 1.0) + contribute(0.0, 1.0, 0.0, 2.0),
                12 => contribute(0.0, 0.0, 2.0, 1.0) + contribute(0.0, 0.0, 1.0, 2.0),
                _ => 0.0,
            };

            // One contribution is a permutation of (1, 1, 1, -1) based on the smaller-sided point
            value += match c2 {
                7 => contribute(1.0, 1.0, 1.0, -1.0),
                11 => contribute(1.0, 1.0, -1.0, 1.0),
                13 => contribute(1.0, -1.0, 1.0, 1.0),
                _ => contribute(-1.0, 1.0, 1.0, 1.0), // c2 == 14
            };
        }

        value
            + contribute(1.0, 1.0, 1.0, 0.0)
            + contribute(1.0, 1.0, 0.0, 1.0)
            + contribute(1.0, 0.0, 1.0, 1.0)
            + contribute(0.0, 1.0, 1.0, 1.0)
            + contribute(1.0, 1.0, 0.0, 0.0)
            + contribute(1.0, 0.0, 1.0, 0.0)
            + contribute(1.0, 0.0, 0.0, 1.0)
            + contribute(0.0, 1.0, 1.0, 0.0)
            + contribute(0.0, 1.0, 0.0, 1.0)
            + contribute(0.0, 0.0, 1.0, 1.0)
    }

    fn inside_first_dispentachoron(
        ins: Vec4<f64>,
        contribute: impl Fn(f64, f64, f64, f64) -> f64,
    ) -> f64 {
        let mut value = 0.0;

        let mut score = Vec2::new(0.0, 0.0);
        let mut point = Vec2::new(0, 0);
        let mut is_bigger_side = Vec2::new(true, true);

        // Decide between (1, 1, 0, 0) and (0, 0, 1, 1)
        if ins.x + ins.y > ins.z + ins.w {
            score.x = ins.x + ins.y;
            point.x = 0x03;
        } else {
            score.x = ins.z + ins.w;
            point.x = 0x0C;
        }

        // Decide between (1, 0, 1, 0) and (0, 1, 0, 1)
        if ins.x + ins.z > ins.y + ins.w {
            score.y = ins.x + ins.z;
            point.y = 0x05;
        } else {
            score.y = ins.y + ins.w;
            point.y = 0x0A;
        }

        // Closer between (1, 0, 0, 1) and (0, 1, 1, 0) will replace the further of a and b, if closer.
        if ins.x + ins.w > ins.y + ins.z {
            let score_value = ins.x + ins.w;
            if score.x >= score.y && score_value > score.y {
                score.y = score_value;
                point.y = 0x09;
            } else if score.x < score.y && score_value > score.x {
                score.x = score_value;
                point.x = 0x09;
            }
        } else {
            let score_value = ins.y + ins.z;
            if score.x >= score.y && score_value > score.y {
                score.y = score_value;
                point.y = 0x06;
            } else if score.x < score.y && score_value > score.x {
                score.x = score_value;
                point.x = 0x06;
            }
        }

        // Decide if (1, 0, 0, 0) is closer.
        let p1 = 2.0 - ins.sum() + ins.x;
        if score.x >= score.y && p1 > score.y {
            score.y = p1;
            point.y = 0x01;
            is_bigger_side.y = false;
        } else if score.x < score.y && p1 > score.x {
            score.x = p1;
            point.x = 0x01;
            is_bigger_side.x = false;
        }

        // Decide if (0, 1, 0, 0) is closer.
        let p2 = 2.0 - ins.sum() + ins.y;
        if score.x >= score.y && p2 > score.y {
            score.y = p2;
            point.y = 0x02;
            is_bigger_side.y = false;
        } else if score.x < score.y && p2 > score.x {
            score.x = p2;
            point.x = 0x02;
            is_bigger_side.x = false;
        }

        // Decide if (0, 0, 1, 0) is closer.
        let p3 = 2.0 - ins.sum() + ins.z;
        if score.x >= score.y && p3 > score.y {
            score.y = p3;
            point.y = 0x04;
            is_bigger_side.y = false;
        } else if score.x < score.y && p3 > score.x {
            score.x = p3;
            point.x = 0x04;
            is_bigger_side.x = false;
        }

        // Decide if (0, 0, 0, 1) is closer.
        let p4 = 2.0 - ins.sum() + ins.w;
        if score.x >= score.y && p4 > score.y {
            point.y = 0x08;
            is_bigger_side.y = false;
        } else if score.x < score.y && p4 > score.x {
            point.x = 0x08;
            is_bigger_side.x = false;
        }

        // Where each of the two closest points are determines how the extra three
        // vertices are calculated.
        if is_bigger_side.x == is_bigger_side.y {
            if is_bigger_side.x {
                // Both closest points on the bigger side
                let c1 = point.x | point.y;
                value += match c1 {
                    7 => contribute(1.0, 1.0, 1.0, 0.0) + contribute(1.0, 1.0, 1.0, -1.0),
                    11 => contribute(1.0, 1.0, 0.0, 1.0) + contribute(1.0, 1.0, -1.0, 1.0),
                    13 => contribute(1.0, 0.0, 1.0, 1.0) + contribute(1.0, -1.0, 1.0, 1.0),
                    _ => contribute(0.0, 1.0, 1.0, 1.0) + contribute(-1.0, 1.0, 1.0, 1.0), // c1 == 14
                };

                // One combination is a permutation of (0, 0, 0, 2) based on c2
                let c2 = point.x & point.y;
                value += match c2 {
                    1 => contribute(2.0, 0.0, 0.0, 0.0),
                    2 => contribute(0.0, 2.0, 0.0, 0.0),
                    4 => contribute(0.0, 0.0, 2.0, 0.0),
                    _ => contribute(0.0, 0.0, 0.0, 2.0), // c2 == 8
                };
            } else {
                // Both closest points on the smaller side
                // One of the two extra points is (0, 0, 0, 0)
                value += contribute(0.0, 0.0, 0.0, 0.0);

                // Other two points are based on the omitted axes.
                let closest = point.x | point.y;
                value += match closest {
                    3 => contribute(1.0, 1.0, -1.0, 0.0) + contribute(1.0, 1.0, 0.0, -1.0),
                    5 => contribute(1.0, -1.0, 1.0, 0.0) + contribute(1.0, 0.0, 1.0, -1.0),
                    6 => contribute(-1.0, 1.0, 1.0, 0.0) + contribute(0.0, 1.0, 1.0, -1.0),
                    9 => contribute(1.0, -1.0, 0.0, 1.0) + contribute(1.0, 0.0, -1.0, 1.0),
                    10 => contribute(-1.0, 1.0, 0.0, 1.0) + contribute(0.0, 1.0, -1.0, 1.0),
                    _ => contribute(-1.0, 0.0, 1.0, 1.0) + contribute(0.0, -1.0, 1.0, 1.0), // closest == 12
                };
            }
        } else {
            // One point on each "side"
            let (c1, c2) = if is_bigger_side.x {
                (point.x, point.y)
            } else {
                (point.y, point.x)
            };

            // Two contributions are the bigger-sided point with each 0 replaced with -1.
            value += match c1 {
                3 => contribute(1.0, 1.0, -1.0, 0.0) + contribute(1.0, 1.0, 0.0, -1.0),
                5 => contribute(1.0, -1.0, 1.0, 0.0) + contribute(1.0, 0.0, 1.0, -1.0),
                6 => contribute(-1.0, 1.0, 1.0, 0.0) + contribute(0.0, 1.0, 1.0, -1.0),
                9 => contribute(1.0, -1.0, 0.0, 1.0) + contribute(1.0, 0.0, -1.0, 1.0),
                10 => contribute(-1.0, 1.0, 0.0, 1.0) + contribute(0.0, 1.0, -1.0, 1.0),
                12 => contribute(-1.0, 0.0, 1.0, 1.0) + contribute(0.0, -1.0, 1.0, 1.0),
                _ => 0.0,
            };

            // One contribution is a permutation of (0, 0, 0, 2) based on the smaller-sided point
            value += match c2 {
                1 => contribute(2.0, 0.0, 0.0, 0.0),
                2 => contribute(0.0, 2.0, 0.0, 0.0),
                4 => contribute(0.0, 0.0, 2.0, 0.0),
                _ => contribute(0.0, 0.0, 0.0, 2.0), // c2 == 8
            };
        }

        value
            + contribute(1.0, 0.0, 0.0, 0.0)
            + contribute(0.0, 1.0, 0.0, 0.0)
            + contribute(0.0, 0.0, 1.0, 0.0)
            + contribute(0.0, 0.0, 0.0, 1.0)
            + contribute(1.0, 1.0, 0.0, 0.0)
            + contribute(1.0, 0.0, 1.0, 0.0)
            + contribute(1.0, 0.0, 0.0, 1.0)
            + contribute(0.0, 1.0, 1.0, 0.0)
            + contribute(0.0, 1.0, 0.0, 1.0)
            + contribute(0.0, 0.0, 1.0, 1.0)
    }

    fn get_value(grid: Vec4<f64>, origin: Vec4<f64>, ins: Vec4<f64>, perm: &PermTable) -> f64 {
        let contribute = |x: f64, y: f64, z: f64, w: f64| {
            utils::contribute::<OpenSimplexNoise4D, Vec4<f64>>(
                Vec4::new(x, y, z, w),
                origin,
                grid,
                perm,
            )
        };

        // Sum those together to get a value that determines the region.
        let in_sum = ins.sum();
        let value = if in_sum <= 1.0 {
            // We're inside the pentachoron (4-Simplex) at (0,0,0,0)
            Self::inside_pentachoron_at_0_0_0_0(ins, contribute)
        } else if in_sum >= 3.0 {
            // We're inside the pentachoron (4-Simplex) at (1, 1, 1, 1)
            Self::inside_pentachoron_at_1_1_1_1(ins, contribute)
        } else if in_sum <= 2.0 {
            // We're inside the first dispentachoron (Rectified 4-Simplex)
            Self::inside_first_dispentachoron(ins, contribute)
        } else {
            // We're inside the second dispentachoron (Rectified 4-Simplex)
            Self::inside_second_dispentachoron(ins, contribute)
        };

        value / NORMALIZING_SCALAR
    }

    fn get_grad_table_index(grid: Vec4<f64>, perm: &PermTable) -> usize {
        let index0 = ((perm[(grid.x as i64 & 0xFF) as usize] + grid.y as i64) & 0xFF) as usize;
        let index1 = ((perm[index0] + grid.z as i64) & 0xFF) as usize;
        ((perm[((perm[index1] + grid.w as i64) & 0xFF) as usize] & 0xFC) >> 2) as usize
    }
}
