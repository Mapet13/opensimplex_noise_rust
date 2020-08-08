use super::{
    utils,
    vector::{vec4::Vec4, VecMethods},
    NoiseEvaluator, PermTable,
};

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
        let point = GRAD_TABLE[OpenSimplexNoise4D::get_grad_table_index(grid, perm)];

        point.x * delta.x + point.y * delta.y + point.z * delta.z + point.w * delta.w
    }

    fn eval(input: Vec4<f64>, perm: &PermTable) -> f64 {
        let stretch: Vec4<f64> = input + (OpenSimplexNoise4D::STRETCH_POINT * input.sum());
        let grid = stretch.map(utils::floor).map(utils::to_f64);

        let squashed: Vec4<f64> = grid + (OpenSimplexNoise4D::SQUISH_POINT * grid.sum());
        let ins = stretch - grid;
        let origin = input - squashed;

        OpenSimplexNoise4D::get_value(grid, origin, ins, perm)
    }
}

impl OpenSimplexNoise4D {
    fn get_value(grid: Vec4<f64>, origin: Vec4<f64>, ins: Vec4<f64>, perm: &PermTable) -> f64 {
        let value = 0.0;
        
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
        if in_sum <= 1.0 {
            // We're inside the pentachoron (4-Simplex) at (0,0,0,0)

            // Determine which two of (0,0,0,1), (0,0,1,0), (0,1,0,0), (1,0,0,0) are closest.
            let mut a_point = 0x01;
            let mut b_point = 0x02;
            let mut a_score = ins.x;
            let mut b_score = ins.y;
            if a_score >= b_score && ins.z > b_score {
                b_score = ins.z;
                b_point = 0x04;
            } else if a_score < b_score && ins.z > a_score {
                a_score = ins.z;
                a_point = 0x04;
            }
            if a_score >= b_score && ins.w > b_score {
                b_score = ins.w;
                b_point = 0x08;
            } else if a_score < b_score && ins.w > a_score {
                a_score = ins.w;
                a_point = 0x08;
            }

            // Now we determine the three lattice points not part of the pentachoron that may contribute.
            // This depends on the closest two pentachoron vertices, including (0, 0, 0, 0)
            let uins = 1.0 - in_sum;
            if uins > a_score || uins > b_score {
                // (0, 0, 0, 0) is one of the closest two pentachoron vertices.
                // Our other closest vertex is the closest out of a and b.
                let closest = if b_score > a_score { b_point } else { a_point };
                if closest == 1 {
                    contribute(1.0, -1.0, 0.0, 0.0);
                    contribute(1.0, 0.0, -1.0, 0.0);
                    contribute(1.0, 0.0, 0.0, -1.0);
                } else if closest == 2 {
                    contribute(-1.0, 1.0, 0.0, 0.0);
                    contribute(0.0, 1.0, -1.0, 0.0);
                    contribute(0.0, 1.0, 0.0, -1.0);
                } else if closest == 4 {
                    contribute(-1.0, 0.0, 1.0, 0.0);
                    contribute(0.0, -1.0, 1.0, 0.0);
                    contribute(0.0, 0.0, 1.0, -1.0);
                } else {
                    // closest == 8
                    contribute(-1.0, 0.0, 0.0, 1.0);
                    contribute(0.0, -1.0, 0.0, 1.0);
                    contribute(0.0, 0.0, -1.0, 1.0);
                }
            } else {
                // (0, 0, 0, 0) is not one of the closest two pentachoron vertices.
                // Our three extra vertices are determined by the closest two.
                let closest = a_point | b_point;
                if closest == 3 {
                    contribute(1.0, 1.0, 0.0, 0.0);
                    contribute(1.0, 1.0, -1.0, 0.0);
                    contribute(1.0, 1.0, 0.0, -1.0);
                } else if closest == 5 {
                    contribute(1.0, 0.0, 1.0, 0.0);
                    contribute(1.0, -1.0, 1.0, 0.0);
                    contribute(1.0, 0.0, 1.0, -1.0);
                } else if closest == 6 {
                    contribute(0.0, 1.0, 1.0, 0.0);
                    contribute(-1.0, 1.0, 1.0, 0.0);
                    contribute(0.0, 1.0, 1.0, -1.0);
                } else if closest == 9 {
                    contribute(1.0, 0.0, 0.0, 1.0);
                    contribute(1.0, -1.0, 0.0, 1.0);
                    contribute(1.0, 0.0, -1.0, 1.0);
                } else if closest == 10 {
                    contribute(0.0, 1.0, 0.0, 1.0);
                    contribute(-1.0, 1.0, 0.0, 1.0);
                    contribute(0.0, 1.0, -1.0, 1.0);
                } else {
                    // closest == 12
                    contribute(0.0, 0.0, 1.0, 1.0);
                    contribute(-1.0, 0.0, 1.0, 1.0);
                    contribute(0.0, -1.0, 1.0, 1.0);
                }
            }

            contribute(0.0, 0.0, 0.0, 0.0);
            contribute(1.0, 0.0, 0.0, 0.0);
            contribute(0.0, 1.0, 0.0, 0.0);
            contribute(0.0, 0.0, 1.0, 0.0);
            contribute(0.0, 0.0, 0.0, 1.0);
        } else if in_sum >= 3.0 {
            // We're inside the pentachoron (4-Simplex) at (1, 1, 1, 1)
            // Determine which two of (1, 1, 1, 0), (1, 1, 0, 1), (1, 0, 1, 1), (0, 1, 1, 1) are closest.
            let mut a_point = 0x0E;
            let mut b_point = 0x0D;
            let mut a_score = ins.x;
            let mut b_score = ins.y;
            if a_score <= b_score && ins.z < b_score {
                b_score = ins.z;
                b_point = 0x0B;
            } else if a_score > b_score && ins.z < a_score {
                a_score = ins.z;
                a_point = 0x0B;
            }
            if a_score <= b_score && ins.w < b_score {
                b_score = ins.w;
                b_point = 0x07;
            } else if a_score > b_score && ins.w < a_score {
                a_score = ins.w;
                a_point = 0x07;
            }

            // Now we determine the three lattice points not part of the pentachoron that may contribute.
            // This depends on the closest two pentachoron vertices, including (0, 0, 0, 0)
            let uins = 4.0 - in_sum;
            if uins < a_score || uins < b_score {
                // (1, 1, 1, 1) is one of the closest two pentachoron vertices.
                // Our other closest vertex is the closest out of a and b.
                let closest = if b_score < a_score { b_point } else { a_point };
                if closest == 7 {
                    contribute(2.0, 1.0, 1.0, 0.0);
                    contribute(1.0, 2.0, 1.0, 0.0);
                    contribute(1.0, 1.0, 2.0, 0.0);
                } else if closest == 11 {
                    contribute(2.0, 1.0, 0.0, 1.0);
                    contribute(1.0, 2.0, 0.0, 1.0);
                    contribute(1.0, 1.0, 0.0, 2.0);
                } else if closest == 13 {
                    contribute(2.0, 0.0, 1.0, 1.0);
                    contribute(1.0, 0.0, 2.0, 1.0);
                    contribute(1.0, 0.0, 1.0, 2.0);
                } else {
                    // closest == 14
                    contribute(0.0, 2.0, 1.0, 1.0);
                    contribute(0.0, 1.0, 2.0, 1.0);
                    contribute(0.0, 1.0, 1.0, 2.0);
                }
            } else {
                // (1,1,1,1) is not one of the closest two pentachoron vertices.
                // Our three extra vertices are determined by the closest two.
                let closest = a_point & b_point;
                if closest == 3 {
                    contribute(1.0, 1.0, 0.0, 0.0);
                    contribute(2.0, 1.0, 0.0, 0.0);
                    contribute(1.0, 2.0, 0.0, 0.0);
                } else if closest == 5 {
                    contribute(1.0, 0.0, 1.0, 0.0);
                    contribute(2.0, 0.0, 1.0, 0.0);
                    contribute(1.0, 0.0, 2.0, 0.0);
                } else if closest == 6 {
                    contribute(0.0, 1.0, 1.0, 0.0);
                    contribute(0.0, 2.0, 1.0, 0.0);
                    contribute(0.0, 1.0, 2.0, 0.0);
                } else if closest == 9 {
                    contribute(1.0, 0.0, 0.0, 1.0);
                    contribute(2.0, 0.0, 0.0, 1.0);
                    contribute(1.0, 0.0, 0.0, 2.0);
                } else if closest == 10 {
                    contribute(0.0, 1.0, 0.0, 1.0);
                    contribute(0.0, 2.0, 0.0, 1.0);
                    contribute(0.0, 1.0, 0.0, 2.0);
                } else {
                    // closest == 12
                    contribute(0.0, 0.0, 1.0, 1.0);
                    contribute(0.0, 0.0, 2.0, 1.0);
                    contribute(0.0, 0.0, 1.0, 2.0);
                }
            }

            contribute(1.0, 1.0, 1.0, 0.0);
            contribute(1.0, 1.0, 0.0, 1.0);
            contribute(1.0, 0.0, 1.0, 1.0);
            contribute(0.0, 1.0, 1.0, 1.0);
            contribute(1.0, 1.0, 1.0, 1.0);
        } else if in_sum <= 2.0 {
            // We're inside the first dispentachoron (Rectified 4-Simplex)
            let mut a_score;
            let mut b_score;
            let mut a_point;
            let mut b_point;
            let mut a_is_bigger_side = true;
            let mut b_is_bigger_side = true;

            // Decide between (1, 1, 0, 0) and (0, 0, 1, 1)
            if ins.x + ins.y > ins.z + ins.w {
                a_score = ins.x + ins.y;
                a_point = 0x03;
            } else {
                a_score = ins.z + ins.w;
                a_point = 0x0C;
            }

            // Decide between (1, 0, 1, 0) and (0, 1, 0, 1)
            if ins.x + ins.z > ins.y + ins.w {
                b_score = ins.x + ins.z;
                b_point = 0x05;
            } else {
                b_score = ins.y + ins.w;
                b_point = 0x0A;
            }

            // Closer between (1, 0, 0, 1) and (0, 1, 1, 0) will replace the further of a and b, if closer.
            if ins.x + ins.w > ins.y + ins.z {
                let score = ins.x + ins.w;
                if a_score >= b_score && score > b_score {
                    b_score = score;
                    b_point = 0x09;
                } else if a_score < b_score && score > a_score {
                    a_score = score;
                    a_point = 0x09;
                }
            } else {
                let score = ins.y + ins.z;
                if a_score >= b_score && score > b_score {
                    b_score = score;
                    b_point = 0x06;
                } else if a_score < b_score && score > a_score {
                    a_score = score;
                    a_point = 0x06;
                }
            }

            // Decide if (1, 0, 0, 0) is closer.
            let p1 = 2.0 - in_sum + ins.x;
            if a_score >= b_score && p1 > b_score {
                b_score = p1;
                b_point = 0x01;
                b_is_bigger_side = false;
            } else if a_score < b_score && p1 > a_score {
                a_score = p1;
                a_point = 0x01;
                a_is_bigger_side = false;
            }

            // Decide if (0, 1, 0, 0) is closer.
            let p2 = 2.0 - in_sum + ins.y;
            if a_score >= b_score && p2 > b_score {
                b_score = p2;
                b_point = 0x02;
                b_is_bigger_side = false;
            } else if a_score < b_score && p2 > a_score {
                a_score = p2;
                a_point = 0x02;
                a_is_bigger_side = false;
            }

            // Decide if (0, 0, 1, 0) is closer.
            let p3 = 2.0 - in_sum + ins.z;
            if a_score >= b_score && p3 > b_score {
                b_score = p3;
                b_point = 0x04;
                b_is_bigger_side = false;
            } else if a_score < b_score && p3 > a_score {
                a_score = p3;
                a_point = 0x04;
                a_is_bigger_side = false;
            }

            // Decide if (0, 0, 0, 1) is closer.
            let p4 = 2.0 - in_sum + ins.w;
            if a_score >= b_score && p4 > b_score {
                b_point = 0x08;
                b_is_bigger_side = false;
            } else if a_score < b_score && p4 > a_score {
                a_point = 0x08;
                a_is_bigger_side = false;
            }

            // Where each of the two closest points are determines how the extra three
            // vertices are calculated.
            if a_is_bigger_side == b_is_bigger_side {
                if a_is_bigger_side {
                    // Both closest points on the bigger side
                    let c1 = a_point | b_point;
                    if c1 == 7 {
                        contribute(1.0, 1.0, 1.0, 0.0);
                        contribute(1.0, 1.0, 1.0, -1.0);
                    } else if c1 == 11 {
                        contribute(1.0, 1.0, 0.0, 1.0);
                        contribute(1.0, 1.0, -1.0, 1.0);
                    } else if c1 == 13 {
                        contribute(1.0, 0.0, 1.0, 1.0);
                        contribute(1.0, -1.0, 1.0, 1.0);
                    } else {
                        // c1 == 14
                        contribute(0.0, 1.0, 1.0, 1.0);
                        contribute(-1.0, 1.0, 1.0, 1.0);
                    }

                    // One combination is a permutation of (0, 0, 0, 2) based on c2
                    let c2 = a_point & b_point;
                    if c2 == 1 {
                        contribute(2.0, 0.0, 0.0, 0.0);
                    } else if c2 == 2 {
                        contribute(0.0, 2.0, 0.0, 0.0);
                    } else if c2 == 4 {
                        contribute(0.0, 0.0, 2.0, 0.0);
                    } else {
                        // c2 == 8
                        contribute(0.0, 0.0, 0.0, 2.0);
                    }
                } else {
                    // Both closest points on the smaller side
                    // One of the two extra points is (0, 0, 0, 0)
                    contribute(0.0, 0.0, 0.0, 0.0);

                    // Other two points are based on the omitted axes.
                    let closest = a_point | b_point;
                    if closest == 3 {
                        contribute(1.0, 1.0, -1.0, 0.0);
                        contribute(1.0, 1.0, 0.0, -1.0);
                    } else if closest == 5 {
                        contribute(1.0, -1.0, 1.0, 0.0);
                        contribute(1.0, 0.0, 1.0, -1.0);
                    } else if closest == 6 {
                        contribute(-1.0, 1.0, 1.0, 0.0);
                        contribute(0.0, 1.0, 1.0, -1.0);
                    } else if closest == 9 {
                        contribute(1.0, -1.0, 0.0, 1.0);
                        contribute(1.0, 0.0, -1.0, 1.0);
                    } else if closest == 10 {
                        contribute(-1.0, 1.0, 0.0, 1.0);
                        contribute(0.0, 1.0, -1.0, 1.0);
                    } else {
                        // closest == 12
                        contribute(-1.0, 0.0, 1.0, 1.0);
                        contribute(0.0, -1.0, 1.0, 1.0);
                    }
                }
            } else {
                // One point on each "side"
                let (c1, c2) = if a_is_bigger_side {
                    (a_point, b_point)
                } else {
                    (b_point, a_point)
                };

                // Two contributions are the bigger-sided point with each 0 replaced with -1.
                if c1 == 3 {
                    contribute(1.0, 1.0, -1.0, 0.0);
                    contribute(1.0, 1.0, 0.0, -1.0);
                } else if c1 == 5 {
                    contribute(1.0, -1.0, 1.0, 0.0);
                    contribute(1.0, 0.0, 1.0, -1.0);
                } else if c1 == 6 {
                    contribute(-1.0, 1.0, 1.0, 0.0);
                    contribute(0.0, 1.0, 1.0, -1.0);
                } else if c1 == 9 {
                    contribute(1.0, -1.0, 0.0, 1.0);
                    contribute(1.0, 0.0, -1.0, 1.0);
                } else if c1 == 10 {
                    contribute(-1.0, 1.0, 0.0, 1.0);
                    contribute(0.0, 1.0, -1.0, 1.0);
                } else if c1 == 12 {
                    contribute(-1.0, 0.0, 1.0, 1.0);
                    contribute(0.0, -1.0, 1.0, 1.0);
                }

                // One contribution is a permutation of (0, 0, 0, 2) based on the smaller-sided point
                if c2 == 1 {
                    contribute(2.0, 0.0, 0.0, 0.0);
                } else if c2 == 2 {
                    contribute(0.0, 2.0, 0.0, 0.0);
                } else if c2 == 4 {
                    contribute(0.0, 0.0, 2.0, 0.0);
                } else {
                    // c2 == 8
                    contribute(0.0, 0.0, 0.0, 2.0);
                }
            }

            contribute(1.0, 0.0, 0.0, 0.0);
            contribute(0.0, 1.0, 0.0, 0.0);
            contribute(0.0, 0.0, 1.0, 0.0);
            contribute(0.0, 0.0, 0.0, 1.0);
            contribute(1.0, 1.0, 0.0, 0.0);
            contribute(1.0, 0.0, 1.0, 0.0);
            contribute(1.0, 0.0, 0.0, 1.0);
            contribute(0.0, 1.0, 1.0, 0.0);
            contribute(0.0, 1.0, 0.0, 1.0);
            contribute(0.0, 0.0, 1.0, 1.0);
        } else {
            // We're inside the second dispentachoron (Rectified 4-Simplex)
            let mut a_score;
            let mut b_score;
            let mut a_point;
            let mut b_point;
            let mut a_is_bigger_side = true;
            let mut b_is_bigger_side = true;

            // Decide between (0,0,1,1) and (1,1,0,0)
            if ins.x + ins.y < ins.z + ins.w {
                a_score = ins.x + ins.y;
                a_point = 0x0C;
            } else {
                a_score = ins.z + ins.w;
                a_point = 0x03;
            }

            // Decide between (0,1,0,1) and (1,0,1,0)
            if ins.x + ins.z < ins.y + ins.w {
                b_score = ins.x + ins.z;
                b_point = 0x0A;
            } else {
                b_score = ins.y + ins.w;
                b_point = 0x05;
            }

            // Closer between (0,1,1,0) and (1,0,0,1) will replace the further of a and b,
            // if closer.
            if ins.x + ins.w < ins.y + ins.z {
                let score = ins.x + ins.w;
                if a_score <= b_score && score < b_score {
                    b_score = score;
                    b_point = 0x06;
                } else if a_score > b_score && score < a_score {
                    a_score = score;
                    a_point = 0x06;
                }
            } else {
                let score = ins.y + ins.z;
                if a_score <= b_score && score < b_score {
                    b_score = score;
                    b_point = 0x09;
                } else if a_score > b_score && score < a_score {
                    a_score = score;
                    a_point = 0x09;
                }
            }

            // Decide if (0, 1, 1, 1) is closer.
            let p1 = 3.0 - in_sum + ins.x;
            if a_score <= b_score && p1 < b_score {
                b_score = p1;
                b_point = 0x0E;
                b_is_bigger_side = false;
            } else if a_score > b_score && p1 < a_score {
                a_score = p1;
                a_point = 0x0E;
                a_is_bigger_side = false;
            }

            // Decide if (1, 0, 1, 1) is closer.
            let p2 = 3.0 - in_sum + ins.y;
            if a_score <= b_score && p2 < b_score {
                b_score = p2;
                b_point = 0x0D;
                b_is_bigger_side = false;
            } else if a_score > b_score && p2 < a_score {
                a_score = p2;
                a_point = 0x0D;
                a_is_bigger_side = false;
            }

            // Decide if (1, 1, 0, 1) is closer.
            let p3 = 3.0 - in_sum + ins.z;
            if a_score <= b_score && p3 < b_score {
                b_score = p3;
                b_point = 0x0B;
                b_is_bigger_side = false;
            } else if a_score > b_score && p3 < a_score {
                a_score = p3;
                a_point = 0x0B;
                a_is_bigger_side = false;
            }

            // Decide if (1, 1, 1, 0) is closer.
            let p4 = 3.0 - in_sum + ins.w;
            if a_score <= b_score && p4 < b_score {
                b_point = 0x07;
                b_is_bigger_side = false;
            } else if a_score > b_score && p4 < a_score {
                a_point = 0x07;
                a_is_bigger_side = false;
            }

            // Where each of the two closest points are determines how the extra three
            // vertices are calculated.
            if a_is_bigger_side == b_is_bigger_side {
                if a_is_bigger_side {
                    // Both closest points on the bigger side

                    // Two contributions are permutations of (0, 0, 0, 1) and (0, 0, 0, 2) based on c1
                    let c1 = a_point & b_point;
                    if c1 == 1 {
                        contribute(1.0, 0.0, 0.0, 0.0);
                        contribute(2.0, 0.0, 0.0, 0.0);
                    } else if c1 == 2 {
                        contribute(0.0, 1.0, 0.0, 0.0);
                        contribute(0.0, 2.0, 0.0, 0.0);
                    } else if c1 == 4 {
                        contribute(0.0, 0.0, 1.0, 0.0);
                        contribute(0.0, 0.0, 2.0, 0.0);
                    } else {
                        // c2 == 8
                        contribute(0.0, 0.0, 0.0, 1.0);
                        contribute(0.0, 0.0, 0.0, 2.0);
                    }

                    // One contribution is a permutation of (1, 1, 1, -1) based on c2
                    let c2 = a_point | b_point;

                    if (c2 & 0x01) == 0 {
                        contribute(-1.0, 1.0, 1.0, 1.0);
                    } else if (c2 & 0x02) == 0 {
                        contribute(1.0, -1.0, 1.0, 1.0);
                    } else if (c2 & 0x04) == 0 {
                        contribute(1.0, 1.0, -1.0, 1.0);
                    } else {
                        // (c2 & 0x08) == 0
                        contribute(1.0, 1.0, 1.0, -1.0);
                    }
                } else {
                    // Both closest points on the smaller side
                    // One of the two extra points is (1, 1, 1, 1)
                    contribute(1.0, 1.0, 1.0, 1.0);

                    // Other two points are based on the shared axes.
                    let closest = a_point & b_point;
                    if closest == 3 {
                        contribute(2.0, 1.0, 0.0, 0.0);
                        contribute(1.0, 2.0, 0.0, 0.0);
                    } else if closest == 5 {
                        contribute(2.0, 0.0, 1.0, 0.0);
                        contribute(1.0, 0.0, 2.0, 0.0);
                    } else if closest == 6 {
                        contribute(0.0, 2.0, 1.0, 0.0);
                        contribute(0.0, 1.0, 2.0, 0.0);
                    } else if closest == 9 {
                        contribute(2.0, 0.0, 0.0, 1.0);
                        contribute(1.0, 0.0, 0.0, 2.0);
                    } else if closest == 10 {
                        contribute(0.0, 2.0, 0.0, 1.0);
                        contribute(0.0, 1.0, 0.0, 2.0);
                    } else {
                        // closest == 12
                        contribute(0.0, 0.0, 2.0, 1.0);
                        contribute(0.0, 0.0, 1.0, 2.0);
                    }
                }
            } else {
                // One point on each "side"
                let (c1, c2) = if a_is_bigger_side {
                    (a_point, b_point)
                } else {
                    (b_point, a_point)
                };

                // Two contributions are the bigger-sided point with each 1 replaced with 2.
                if c1 == 3 {
                    contribute(2.0, 1.0, 0.0, 0.0);
                    contribute(1.0, 2.0, 0.0, 0.0);
                } else if c1 == 5 {
                    contribute(2.0, 0.0, 1.0, 0.0);
                    contribute(1.0, 0.0, 2.0, 0.0);
                } else if c1 == 6 {
                    contribute(0.0, 2.0, 1.0, 0.0);
                    contribute(0.0, 1.0, 2.0, 0.0);
                } else if c1 == 9 {
                    contribute(2.0, 0.0, 0.0, 1.0);
                    contribute(1.0, 0.0, 0.0, 2.0);
                } else if c1 == 10 {
                    contribute(0.0, 2.0, 0.0, 1.0);
                    contribute(0.0, 1.0, 0.0, 2.0);
                } else if c1 == 12 {
                    contribute(0.0, 0.0, 2.0, 1.0);
                    contribute(0.0, 0.0, 1.0, 2.0);
                }

                // One contribution is a permutation of (1, 1, 1, -1) based on the smaller-sided point
                if c2 == 7 {
                    contribute(1.0, 1.0, 1.0, -1.0);
                } else if c2 == 11 {
                    contribute(1.0, 1.0, -1.0, 1.0);
                } else if c2 == 13 {
                    contribute(1.0, -1.0, 1.0, 1.0);
                } else {
                    // c2 == 14
                    contribute(-1.0, 1.0, 1.0, 1.0);
                }
            }

            contribute(1.0, 1.0, 1.0, 0.0);
            contribute(1.0, 1.0, 0.0, 1.0);
            contribute(1.0, 0.0, 1.0, 1.0);
            contribute(0.0, 1.0, 1.0, 1.0);
            contribute(1.0, 1.0, 0.0, 0.0);
            contribute(1.0, 0.0, 1.0, 0.0);
            contribute(1.0, 0.0, 0.0, 1.0);
            contribute(0.0, 1.0, 1.0, 0.0);
            contribute(0.0, 1.0, 0.0, 1.0);
            contribute(0.0, 0.0, 1.0, 1.0);
        }

        value / NORMALIZING_SCALAR
    }

    fn get_grad_table_index(grid: Vec4<f64>, perm: &PermTable) -> usize {
        let index0 = ((perm[(grid.x as i64 & 0xFF) as usize] + grid.y as i64) & 0xFF) as usize;
        let index1 = ((perm[index0] + grid.z as i64) & 0xFF) as usize;
        ((perm[((perm[index1] + grid.w as i64) & 0xFF) as usize] & 0xFC) >> 2) as usize
    }
}
