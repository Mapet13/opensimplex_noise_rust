use super::constants::PSIZE;
use super::vector::{vec3::Vec3, VecTrait};

use super::utils;
use super::NoiseEvaluator;

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
}

impl OpenSimplexNoise3D {
    pub fn eval_3d(x: f64, y: f64, z: f64, perm: &[i64; PSIZE as usize]) -> f64 {
        let input = Vec3::new(x, y, z);
        let stretch: Vec3<f64> = input + (OpenSimplexNoise3D::STRETCH_POINT * input.sum());
        let grid = stretch.map(utils::fast_floor).map(utils::to_f64);

        let squashed: Vec3<f64> = grid + (OpenSimplexNoise3D::SQUISH_POINT * grid.sum());
        let ins = stretch - grid;
        let origin = input - squashed;

        OpenSimplexNoise3D::eval(grid, origin, ins, perm)
    }

    fn eval(
        grid: Vec3<f64>,
        origin: Vec3<f64>,
        ins: Vec3<f64>,
        perm: &[i64; PSIZE as usize],
    ) -> f64 {
        let mut value = 0.0;
        let mut contribute = |dx: f64, dy: f64, dz: f64| {
            value += utils::contribute::<OpenSimplexNoise3D, Vec3<f64>>(
                Vec3::new(dx, dy, dz),
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
            let mut a_score = ins.x;
            let mut b_score = ins.y;
            let mut a_point = 0x01;
            let mut b_point = 0x02;
            if ins.x >= ins.y && ins.z > ins.y {
                b_score = ins.z;
                b_point = 0x04;
            } else if ins.x < ins.y && ins.z > ins.x {
                a_score = ins.z;
                a_point = 0x04;
            }

            // Now we determine the two lattice points not part of the tetrahedron that may contribute.
            // This depends on the closest two tetrahedral vertices, including (0, 0, 0)
            let wins = 1.0 - in_sum;
            if wins > a_score || wins > b_score {
                // (0, 0, 0) is one of the closest two tetrahedral vertices.
                // Our other closest vertex is the closest out of a and b.
                let closest = if b_score > a_score { b_point } else { a_point };
                if closest == 1 {
                    contribute(1.0, -1.0, 0.0);
                    contribute(1.0, 0.0, -1.0);
                } else if closest == 2 {
                    contribute(-1.0, 1.0, 0.0);
                    contribute(0.0, 1.0, -1.0);
                    // closest == 4
                    contribute(-1.0, 0.0, 1.0);
                    contribute(0.0, -1.0, 1.0);
                }
            } else {
                // (0, 0, 0) is not one of the closest two tetrahedral vertices.
                // Our two extra vertices are determined by the closest two.
                let closest = a_point | b_point;
                if closest == 3 {
                    contribute(1.0, 1.0, 0.0);
                    contribute(1.0, 1.0, -1.0);
                } else if closest == 5 {
                    contribute(1.0, 0.0, 1.0);
                    contribute(1.0, -1.0, 1.0);
                } else {
                    // closest == 6
                    contribute(0.0, 1.0, 1.0);
                    contribute(-1.0, 1.0, 1.0);
                }
            }

            contribute(0.0, 0.0, 0.0);
            contribute(1.0, 0.0, 0.0);
            contribute(0.0, 1.0, 0.0);
            contribute(0.0, 0.0, 1.0);
        } else if in_sum >= 2.0 {
            // Inside the tetrahedron (3-Simplex) at (1, 1, 1)

            // Determine which two tetrahedral vertices are the closest, out of (1, 1, 0), (1, 0, 1), (0, 1, 1) but not (1, 1, 1).
            let mut a_point = 0x06;
            let mut a_score = ins.x;
            let mut b_point = 0x05;
            let mut b_score = ins.y;
            if a_score <= b_score && ins.z < b_score {
                b_score = ins.z;
                b_point = 0x03;
            } else if a_score > b_score && ins.z < a_score {
                a_score = ins.z;
                a_point = 0x03;
            }

            // Now we determine the two lattice points not part of the tetrahedron that may contribute.
            // This depends on the closest two tetrahedral vertices, including (1, 1, 1)
            let wins = 3.0 - in_sum;
            if wins < a_score || wins < b_score {
                // (1, 1, 1) is one of the closest two tetrahedral vertices.
                // Our other closest vertex is the closest out of a and b.
                let closest = if b_score < a_score { b_point } else { a_point };
                if closest == 3 {
                    contribute(2.0, 1.0, 0.0);
                    contribute(1.0, 2.0, 0.0);
                } else if closest == 5 {
                    contribute(2.0, 0.0, 1.0);
                    contribute(1.0, 0.0, 2.0);
                } else {
                    // closest == 6
                    contribute(0.0, 2.0, 1.0);
                    contribute(0.0, 1.0, 2.0);
                }
            } else {
                // (1, 1, 1) is not one of the closest two tetrahedral vertices.
                // Our two extra vertices are determined by the closest two.
                let closest = a_point & b_point;
                if closest == 1 {
                    contribute(1.0, 0.0, 0.0);
                    contribute(2.0, 0.0, 0.0);
                } else if closest == 2 {
                    contribute(0.0, 1.0, 0.0);
                    contribute(0.0, 2.0, 0.0);
                } else {
                    // closest == 4
                    contribute(0.0, 0.0, 1.0);
                    contribute(0.0, 0.0, 2.0);
                }
            }

            contribute(1.0, 1.0, 0.0);
            contribute(1.0, 0.0, 1.0);
            contribute(0.0, 1.0, 1.0);
            contribute(1.0, 1.0, 1.0);
        } else {
            // Inside the octahedron (Rectified 3-Simplex) in between.
            let a_score;
            let b_score;
            let mut a_point;
            let mut b_point;
            let mut a_is_further_side;
            let mut b_is_further_side;

            // Decide between point (0, 0, 1) and (1, 1, 0) as closest
            let p1 = ins.x + ins.y;
            if p1 > 1.0 {
                a_score = p1 - 1.0;
                a_point = 0x03;
                a_is_further_side = true;
            } else {
                a_score = 1.0 - p1;
                a_point = 0x04;
                a_is_further_side = false;
            }

            // Decide between point (0, 1, 0) and (1, 0, 1) as closest
            let p2 = ins.x + ins.z;
            if p2 > 1.0 {
                b_score = p2 - 1.0;
                b_point = 0x05;
                b_is_further_side = true;
            } else {
                b_score = 1.0 - p2;
                b_point = 0x02;
                b_is_further_side = false;
            }

            // The closest out of the two (1, 0, 0) and (0, 1, 1) will replace
            // the furthest out of the two decided above, if closer.
            let p3 = ins.y + ins.z;
            if p3 > 1.0 {
                let score = p3 - 1.0;
                if a_score <= b_score && a_score < score {
                    a_point = 0x06;
                    a_is_further_side = true;
                } else if a_score > b_score && b_score < score {
                    b_point = 0x06;
                    b_is_further_side = true;
                }
            } else {
                let score = 1.0 - p3;
                if a_score <= b_score && a_score < score {
                    a_point = 0x01;
                    a_is_further_side = false;
                } else if a_score > b_score && b_score < score {
                    b_point = 0x01;
                    b_is_further_side = false;
                }
            }

            // Where each of the two closest points are determines how the extra two vertices are calculated.
            if a_is_further_side == b_is_further_side {
                if a_is_further_side {
                    // Both closest points on (1, 1, 1) side

                    // One of the two extra points is (1, 1, 1)
                    contribute(1.0, 1.0, 1.0);

                    // Other extra point is based on the shared axis.
                    let closest = a_point & b_point;
                    if closest == 1 {
                        contribute(2.0, 0.0, 0.0);
                    } else if closest == 2 {
                        contribute(0.0, 2.0, 0.0);
                    } else {
                        // closest == 4
                        contribute(0.0, 0.0, 2.0);
                    }
                } else {
                    // Both closest points on (0, 0, 0) side

                    // One of the two extra points is (0, 0, 0)
                    contribute(0.0, 0.0, 0.0);

                    // Other extra point is based on the omitted axis.
                    let closest = a_point | b_point;
                    if closest == 3 {
                        contribute(1.0, 1.0, -1.0);
                    } else if closest == 5 {
                        contribute(1.0, -1.0, 1.0);
                    } else {
                        // closest == 6
                        contribute(-1.0, 1.0, 1.0);
                    }
                }
            } else {
                // One point on (0, 0, 0) side, one point on (1, 1, 1) side
                let (c1, c2) = if a_is_further_side {
                    (a_point, b_point)
                } else {
                    (b_point, a_point)
                };

                // One contribution is a permutation of (1, 1, -1)
                if c1 == 3 {
                    contribute(1.0, 1.0, -1.0);
                } else if c1 == 5 {
                    contribute(1.0, -1.0, 1.0);
                } else {
                    // c1 == 6
                    contribute(-1.0, 1.0, 1.0);
                }
                // One contribution is a permutation of (0, 0, 2)
                if c2 == 1 {
                    contribute(2.0, 0.0, 0.0);
                } else if c2 == 2 {
                    contribute(0.0, 2.0, 0.0);
                } else {
                    // c2 == 4
                    contribute(0.0, 0.0, 2.0);
                }
            }

            contribute(1.0, 0.0, 0.0);
            contribute(0.0, 1.0, 0.0);
            contribute(0.0, 0.0, 1.0);
            contribute(1.0, 1.0, 0.0);
            contribute(1.0, 0.0, 1.0);
            contribute(0.0, 1.0, 1.0);
        }

        value / NORMALIZING_SCALAR
    }
}
