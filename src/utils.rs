use super::constants::PSIZE;
use super::vector::{VecType};
use super::NoiseEvaluator;

pub fn contribute<
    NoiseEvaluatorType: NoiseEvaluator<Vec>,
    Vec: VecType<f64>,
>(
    delta: Vec,
    origin: Vec,
    grid: Vec,
    perm: &[i64; PSIZE as usize],
) -> f64 {
    let shifted: Vec = origin - delta - NoiseEvaluatorType::SQUISH_POINT * delta.sum();
    let attn: f64 = 2.0 - shifted.get_attenuation_factor();
    if attn > 0.0 {
        let attn2 = attn * attn;
        return attn2 * attn2 * NoiseEvaluatorType::extrapolate(grid + delta, shifted, perm);
    }

    0.0
}