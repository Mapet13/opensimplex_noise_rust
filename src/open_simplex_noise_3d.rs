use super::utils;
use super::vector::{vec3::Vec3, VecMethods};
use super::NoiseEvaluator;
use super::{vector::vec2::Vec2, PermTable};

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

    fn extrapolate(grid: Vec3<f64>, delta: Vec3<f64>, perm: &PermTable) -> f64 {
        let point = GRAD_TABLE_2D[OpenSimplexNoise3D::get_grad_table_index(grid, perm)];

        point.x * delta.x + point.y * delta.y + point.z * delta.z
    }

    fn eval(input: Vec3<f64>, perm: &PermTable) -> f64 {
        let stretch: Vec3<f64> = input + (OpenSimplexNoise3D::STRETCH_POINT * input.sum());
        let grid = stretch.map(utils::floor).map(utils::to_f64);

        let squashed: Vec3<f64> = grid + (OpenSimplexNoise3D::SQUISH_POINT * grid.sum());
        let ins = stretch - grid;
        let origin = input - squashed;

        OpenSimplexNoise3D::get_value(grid, origin, ins, perm)
    }
}

impl OpenSimplexNoise3D {
    fn get_value(grid: Vec3<f64>, origin: Vec3<f64>, ins: Vec3<f64>, perm: &PermTable) -> f64 {
        let contribute = |x: f64, y: f64, z: f64| {
            utils::contribute::<OpenSimplexNoise3D, Vec3<f64>>(
                Vec3::new(x, y, z),
                origin,
                grid,
                perm,
            )
        };

        // Sum those together to get a value that determines the region.
        let value = match ins.sum() {
            in_sum if in_sum <= 1.0 => {
                // Inside the tetrahedron (3-Simplex) at (0, 0, 0)
                OpenSimplexNoise3D::inside_tetrahedron_at_0_0_0(ins, in_sum, contribute)
            }
            in_sum if in_sum >= 2.0 => {
                // Inside the tetrahedron (3-Simplex) at (1, 1, 1)
                OpenSimplexNoise3D::inside_tetrahedron_at_1_1_1(ins, in_sum, contribute)
            }
            _ => {
                // Inside the octahedron (Rectified 3-Simplex) in between.
                OpenSimplexNoise3D::inside_octahedron_in_between(ins, contribute)
            }
        };

        value / NORMALIZING_SCALAR
    }

    fn inside_tetrahedron_at_0_0_0(
        ins: Vec3<f64>,
        in_sum: f64,
        contribute: impl Fn(f64, f64, f64) -> f64,
    ) -> f64 {
        // Determine which two of (0, 0, 1), (0, 1, 0), (1, 0, 0) are closest.
        let (score, point) = OpenSimplexNoise3D::determine_closest_point(
            Vec2::new(ins.x, ins.y),
            Vec2::new(1, 2),
            Vec2::new(4, 4),
            ins,
        );

        // Now we determine the two lattice points not part of the tetrahedron that may contribute.
        // This depends on the closest two tetrahedral vertices, including (0, 0, 0)
        let value = OpenSimplexNoise3D::determine_lattice_points_including_0_0_0(
            in_sum,
            score,
            point,
            |x, y, z| contribute(x, y, z),
        );

        value
            + contribute(0.0, 0.0, 0.0)
            + contribute(1.0, 0.0, 0.0)
            + contribute(0.0, 1.0, 0.0)
            + contribute(0.0, 0.0, 1.0)
    }

    fn inside_tetrahedron_at_1_1_1(
        ins: Vec3<f64>,
        in_sum: f64,
        contribute: impl Fn(f64, f64, f64) -> f64,
    ) -> f64 {
        // Determine which two tetrahedral vertices are the closest, out of (1, 1, 0), (1, 0, 1), (0, 1, 1) but not (1, 1, 1).
        let (score, point) = OpenSimplexNoise3D::determine_closest_point(
            Vec2::new(ins.x, ins.y),
            Vec2::new(6, 5),
            Vec2::new(3, 3),
            ins,
        );

        // Now we determine the two lattice points not part of the tetrahedron that may contribute.
        // This depends on the closest two tetrahedral vertices, including (1, 1, 1)
        let value = OpenSimplexNoise3D::determine_lattice_points_including_1_1_1(
            in_sum,
            score,
            point,
            |x, y, z| contribute(x, y, z),
        );

        value
            + contribute(1.0, 1.0, 0.0)
            + contribute(1.0, 0.0, 1.0)
            + contribute(0.0, 1.0, 1.0)
            + contribute(1.0, 1.0, 1.0)
    }

    fn inside_octahedron_in_between(
        ins: Vec3<f64>,
        contribute: impl Fn(f64, f64, f64) -> f64,
    ) -> f64 {
        let (is_further_side, point) = OpenSimplexNoise3D::determine_further_side(ins);

        // Where each of the two closest points are determines how the extra two vertices are calculated.
        let value = if is_further_side.x == is_further_side.y {
            if is_further_side.x {
                // Both closest points on (1, 1, 1) side
                // One of the two extra points is (1, 1, 1)
                // Other extra point is based on the shared axis.
                let closest = point.x & point.y;

                contribute(1.0, 1.0, 1.0)
                    + match closest {
                        1 => contribute(2.0, 0.0, 0.0),
                        2 => contribute(0.0, 2.0, 0.0),
                        _ => contribute(0.0, 0.0, 2.0), // closest == 4
                    }
            } else {
                // Both closest points on (0, 0, 0) side
                // One of the two extra points is (0, 0, 0)
                // Other extra point is based on the omitted axis.
                let closest = point.x | point.y;

                contribute(0.0, 0.0, 0.0)
                    + match closest {
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
            // One contribution is a permutation of (0, 0, 2)
            (match c1 {
                3 => contribute(1.0, 1.0, -1.0),
                5 => contribute(1.0, -1.0, 1.0),
                _ => contribute(-1.0, 1.0, 1.0), // c1 == 6
            }) + match c2 {
                1 => contribute(2.0, 0.0, 0.0),
                2 => contribute(0.0, 2.0, 0.0),
                _ => contribute(0.0, 0.0, 2.0), // c1 == 4
            }
        };

        value
            + contribute(1.0, 0.0, 0.0)
            + contribute(0.0, 1.0, 0.0)
            + contribute(0.0, 0.0, 1.0)
            + contribute(1.0, 1.0, 0.0)
            + contribute(1.0, 0.0, 1.0)
            + contribute(0.0, 1.0, 1.0)
    }

    fn decide_between_points(ins: Vec3<f64>) -> (Vec2<f64>, Vec2<i32>, Vec2<bool>) {
        let decide_between_points = |p: f64, point_val: (i32, i32)| {
            if p > 1.0 {
                return (p - 1.0, point_val.0, true);
            }
            (1.0 - p, point_val.1, false)
        };

        // Decide between point (0, 0, 1) and (1, 1, 0) as closest
        let (score_x, point_x, is_further_side_x) = decide_between_points(ins.x + ins.y, (3, 4));
        // Decide between point (0, 1, 0) and (1, 0, 1) as closest
        let (score_y, point_y, is_further_side_y) = decide_between_points(ins.x + ins.z, (5, 2));

        (
            Vec2::new(score_x, score_y),
            Vec2::new(point_x, point_y),
            Vec2::new(is_further_side_x, is_further_side_y),
        )
    }

    fn determine_further_side(ins: Vec3<f64>) -> (Vec2<bool>, Vec2<i32>) {
        let (score, mut point, mut is_further_side) =
            OpenSimplexNoise3D::decide_between_points(ins);

        // The closest out of the two (1, 0, 0) and (0, 1, 1) will replace
        // the furthest out of the two decided above, if closer.
        let p = ins.y + ins.z;
        if p > 1.0 {
            let score_value = p - 1.0;
            if score.x <= score.y && score.x < score_value {
                point.x = 6;
                is_further_side.x = true;
            } else if score.x > score.y && score.y < score_value {
                point.y = 6;
                is_further_side.y = true;
            }
        } else {
            let score_value = 1.0 - p;
            if score.x <= score.y && score.x < score_value {
                point.x = 1;
                is_further_side.x = false;
            } else if score.x > score.y && score.y < score_value {
                point.y = 1;
                is_further_side.y = false;
            }
        }

        (is_further_side, point)
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

    fn determine_lattice_points_including_0_0_0(
        in_sum: f64,
        score: Vec2<f64>,
        point: Vec2<i64>,
        contribute: impl Fn(f64, f64, f64) -> f64,
    ) -> f64 {
        let wins = 1.0 - in_sum;

        if wins > score.x || wins > score.y {
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
        }
    }

    fn determine_lattice_points_including_1_1_1(
        in_sum: f64,
        score: Vec2<f64>,
        point: Vec2<i64>,
        contribute: impl Fn(f64, f64, f64) -> f64,
    ) -> f64 {
        let wins = 3.0 - in_sum;
        if wins < score.x || wins < score.y {
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
        }
    }

    fn get_grad_table_index(grid: Vec3<f64>, perm: &PermTable) -> usize {
        let index0 = ((perm[(grid.x as i64 & 0xFF) as usize] + grid.y as i64) & 0xFF) as usize;
        let index1 = ((perm[index0] + grid.z as i64) & 0xFF) as usize;
        perm[index1] as usize % GRAD_TABLE_2D.len()
    }
}
