use super::constants::PSIZE;
use super::vector::{vec3::Vec3, VecMethods};

use super::utils;
use super::NoiseEvaluator;
use crate::vector::vec2::Vec2;

const STRETCH: f64 = -1.0 / 6.0; // (1 / sqrt(3 + 1) - 1) / 3
const SQUISH: f64 = 1.0 / 3.0; // (sqrt(3 + 1) - 1) / 3

const NORMALIZING_SCALAR: f64 = 103.0;

const GRAD_TABLE_2D: [Vec3<f64>; 24] = [
    Vec3::new(-11.0, 4.0, 4.0),
    Vec3::new(-4.0, 11.0, 4.0),
    Vec3::new(-4.0, 4.0, 11.0),
    Vec3::new(11.0, 4.0, 4.0),
    Vec3::new(4.0, 11.0, 4.0),
    Vec3::new(4.0, 4.0, 11.0),
    Vec3::new(-11.0, -4.0, 4.0),
    Vec3::new(-4.0, -11.0, 4.0),
    Vec3::new(-4.0, -4.0, 11.0),
    Vec3::new(11.0, -4.0, 4.0),
    Vec3::new(4.0, -11.0, 4.0),
    Vec3::new(4.0, -4.0, 11.0),
    Vec3::new(-11.0, 4.0, -4.0),
    Vec3::new(-4.0, 11.0, -4.0),
    Vec3::new(-4.0, 4.0, -11.0),
    Vec3::new(11.0, 4.0, -4.0),
    Vec3::new(4.0, 11.0, -4.0),
    Vec3::new(4.0, 4.0, -11.0),
    Vec3::new(-11.0, -4.0, -4.0),
    Vec3::new(-4.0, -11.0, -4.0),
    Vec3::new(-4.0, -4.0, -11.0),
    Vec3::new(11.0, -4.0, -4.0),
    Vec3::new(4.0, -11.0, -4.0),
    Vec3::new(4.0, -4.0, -11.0),
];

pub struct OpenSimplexNoise3D {}

impl NoiseEvaluator<Vec3<f64>> for OpenSimplexNoise3D {
    const STRETCH_POINT: Vec3<f64> = Vec3::new(STRETCH, STRETCH, STRETCH);
    const SQUISH_POINT: Vec3<f64> = Vec3::new(SQUISH, SQUISH, SQUISH);

    fn extrapolate(grid: Vec3<f64>, delta: Vec3<f64>, perm: &[i64; PSIZE as usize]) -> f64 {
        let index0 = (perm[(grid.x as i64 & 0xFF) as usize] + grid.y as i64) & 0xFF;
        let index1 = (perm[index0 as usize] + grid.z as i64) & 0xFF;
        let index2 = perm[index1 as usize] % GRAD_TABLE_2D.len() as i64;
        let point = GRAD_TABLE_2D[index2 as usize];

        point.x * delta.x + point.y * delta.y + point.z * delta.z
    }

    fn eval(input: Vec3<f64>, perm: &[i64; PSIZE as usize]) -> f64 {
        let stretch: Vec3<f64> = input + (OpenSimplexNoise3D::STRETCH_POINT * input.sum());
        let grid = stretch.map(utils::floor).map(utils::to_f64);

        let squashed: Vec3<f64> = grid + (OpenSimplexNoise3D::SQUISH_POINT * grid.sum());
        let ins = stretch - grid;
        let origin = input - squashed;

        OpenSimplexNoise3D::get_value(grid, origin, ins, perm)
    }
}

fn determine_closest_point(
    score: Vec2<f64>,
    point: Vec2<i64>,
    factor: Vec2<i64>,
    ins: Vec3<f64>,
) -> (Vec2<f64>, Vec2<i64>) {
    let mut score = score;
    let mut point = point;
    if ins.x >= ins.y && ins.z > ins.y {
        score.y = ins.z;
        point.y = factor.y;
    } else if ins.x < ins.y && ins.z > ins.x {
        score.x = ins.z;
        point.x = factor.x;
    }

    (score, point)
}

impl OpenSimplexNoise3D {
    fn get_value(
        grid: Vec3<f64>,
        origin: Vec3<f64>,
        ins: Vec3<f64>,
        perm: &[i64; PSIZE as usize],
    ) -> f64 {
        let mut value = 0.0;
        let contribute = |x: f64, y: f64, z: f64| {
            utils::contribute::<OpenSimplexNoise3D, Vec3<f64>>(
                Vec3::new(x, y, z),
                origin,
                grid,
                perm,
            )
        };

        // Sum those together to get a value that determines the region.
        let in_sum = ins.sum();
        if in_sum <= 1.0 {
            // Inside the tetrahedron (3-Simplex) at (0, 0, 0)

            // Determine which two of (0, 0, 1), (0, 1, 0), (1, 0, 0) are closest.
            let (score, point) = determine_closest_point(
                Vec2::new(ins.x, ins.y),
                Vec2::new(1, 2),
                Vec2::new(4, 4),
                ins,
            );

            // Now we determine the two lattice points not part of the tetrahedron that may contribute.
            // This depends on the closest two tetrahedral vertices, including (0, 0, 0)

            let wins = 1.0 - in_sum;
            value += if wins > score.x || wins > score.y {
                // (0, 0, 0) is one of the closest two tetrahedral vertices.
                // Our other closest vertex is the closest out of a and b.
                let closest = if score.y > score.x { point.y } else { point.x };
                match closest {
                    1 => contribute(1.0, -1.0, 0.0) + contribute(1.0, 0.0, -1.0),
                    2 => contribute(-1.0, 1.0, 0.0) + contribute(0.0, 1.0, -1.0),
                    _ => contribute(-1.0, 0.0, 1.0) + contribute(0.0, -1.0, 1.0), // closest == 4
                }
            } else {
                // (0, 0, 0) is not one of the closest two tetrahedral vertices.
                // Our two extra vertices are determined by the closest two.
                let closest = point.x | point.y;
                match closest {
                    3 => contribute(1.0, 1.0, 0.0) + contribute(1.0, 1.0, -1.0),
                    5 => contribute(1.0, 0.0, 1.0) + contribute(1.0, -1.0, 1.0),
                    _ => contribute(0.0, 1.0, 1.0) + contribute(-1.0, 1.0, 1.0), // closest == 6
                }
            };

            value += contribute(0.0, 0.0, 0.0)
                + contribute(1.0, 0.0, 0.0)
                + contribute(0.0, 1.0, 0.0)
                + contribute(0.0, 0.0, 1.0);
        } else if in_sum >= 2.0 {
            // Inside the tetrahedron (3-Simplex) at (1, 1, 1)

            // Determine which two tetrahedral vertices are the closest, out of (1, 1, 0), (1, 0, 1), (0, 1, 1) but not (1, 1, 1).
            let (score, point) = determine_closest_point(
                Vec2::new(ins.x, ins.y),
                Vec2::new(6, 5),
                Vec2::new(3, 3),
                ins,
            );

            // Now we determine the two lattice points not part of the tetrahedron that may contribute.
            // This depends on the closest two tetrahedral vertices, including (1, 1, 1)
            let wins = 3.0 - in_sum;
            value += if wins < score.x || wins < score.y {
                // (1, 1, 1) is one of the closest two tetrahedral vertices.
                // Our other closest vertex is the closest out of a and b.
                let closest = if score.y < score.x { point.y } else { point.x };
                match closest {
                    3 => contribute(2.0, 1.0, 0.0) + contribute(1.0, 2.0, 0.0),
                    5 => contribute(2.0, 0.0, 1.0) + contribute(1.0, 0.0, 2.0),
                    _ => contribute(0.0, 2.0, 1.0) + contribute(0.0, 1.0, 2.0), // closest == 6
                }
            } else {
                // (1, 1, 1) is not one of the closest two tetrahedral vertices.
                // Our two extra vertices are determined by the closest two.
                let closest = point.x & point.y;
                match closest {
                    1 => contribute(1.0, 0.0, 0.0) + contribute(2.0, 0.0, 0.0),
                    2 => contribute(0.0, 1.0, 0.0) + contribute(0.0, 2.0, 0.0),
                    _ => contribute(0.0, 0.0, 1.0) + contribute(0.0, 0.0, 2.0), // closest == 4
                }
            };

            value += contribute(1.0, 1.0, 0.0)
                + contribute(1.0, 0.0, 1.0)
                + contribute(0.0, 1.0, 1.0)
                + contribute(1.0, 1.0, 1.0);
        } else {
            // Inside the octahedron (Rectified 3-Simplex) in between.
            let mut score = Vec2::new(0.0, 0.0);
            let mut point = Vec2::new(0, 0);
            let mut is_further_side = Vec2::new(false, false);

            // Decide between point (0, 0, 1) and (1, 1, 0) as closest
            let p1 = ins.x + ins.y;
            if p1 > 1.0 {
                score.x = p1 - 1.0;
                point.x = 3;
                is_further_side.x = true;
            } else {
                score.x = 1.0 - p1;
                point.x = 4;
                is_further_side.x = false;
            }

            // Decide between point (0, 1, 0) and (1, 0, 1) as closest
            let p2 = ins.x + ins.z;
            if p2 > 1.0 {
                score.y = p2 - 1.0;
                point.y = 5;
                is_further_side.y = true;
            } else {
                score.y = 1.0 - p2;
                point.y = 2;
                is_further_side.y = false;
            }

            // The closest out of the two (1, 0, 0) and (0, 1, 1) will replace
            // the furthest out of the two decided above, if closer.
            let p3 = ins.y + ins.z;
            if p3 > 1.0 {
                let score_value = p3 - 1.0;
                if score.x <= score.y && score.x < score_value {
                    point.x = 6;
                    is_further_side.x = true;
                } else if score.x > score.y && score.y < score_value {
                    point.y = 6;
                    is_further_side.y = true;
                }
            } else {
                let score_value = 1.0 - p3;
                if score.x <= score.y && score.x < score_value {
                    point.x = 1;
                    is_further_side.x = false;
                } else if score.x > score.y && score.y < score_value {
                    point.y = 1;
                    is_further_side.y = false;
                }
            }

            // Where each of the two closest points are determines how the extra two vertices are calculated.
            if is_further_side.x == is_further_side.y {
                if is_further_side.x {
                    // Both closest points on (1, 1, 1) side

                    // One of the two extra points is (1, 1, 1)
                    value += contribute(1.0, 1.0, 1.0);

                    // Other extra point is based on the shared axis.
                    let closest = point.x & point.y;
                    value += match closest {
                        1 => contribute(2.0, 0.0, 0.0),
                        2 => contribute(0.0, 2.0, 0.0),
                        _ => contribute(0.0, 0.0, 2.0), // closest == 4
                    }
                } else {
                    // Both closest points on (0, 0, 0) side

                    // One of the two extra points is (0, 0, 0)
                    value += contribute(0.0, 0.0, 0.0);

                    // Other extra point is based on the omitted axis.
                    let closest = point.x | point.y;
                    value += match closest {
                        3 => contribute(1.0, 1.0, -1.0),
                        4 => contribute(1.0, -1.0, 1.0),
                        _ => contribute(-1.0, 1.0, 1.0), // closest == 6
                    }
                }
            } else {
                // One point on (0, 0, 0) side, one point on (1, 1, 1) side
                let (c1, c2) = if is_further_side.x {
                    (point.x, point.y)
                } else {
                    (point.y, point.x)
                };

                // One contribution is a permutation of (1, 1, -1)
                value += match c1 {
                    3 => contribute(1.0, 1.0, -1.0),
                    5 => contribute(1.0, -1.0, 1.0),
                    _ => contribute(-1.0, 1.0, 1.0), // c1 == 6
                };
                // One contribution is a permutation of (0, 0, 2)
                value += match c2 {
                    1 => contribute(2.0, 0.0, 0.0),
                    2 => contribute(0.0, 2.0, 0.0),
                    _ => contribute(0.0, 0.0, 2.0), // c1 == 4
                };
            }

            value += contribute(1.0, 0.0, 0.0)
                + contribute(0.0, 1.0, 0.0)
                + contribute(0.0, 0.0, 1.0)
                + contribute(1.0, 1.0, 0.0)
                + contribute(1.0, 0.0, 1.0)
                + contribute(0.0, 1.0, 1.0);
        }

        value / NORMALIZING_SCALAR
    }
}
