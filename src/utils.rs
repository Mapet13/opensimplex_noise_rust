use super::vector::VecType;
use super::NoiseEvaluator;
use super::PermTable;

pub fn contribute<NoiseEvaluatorType: NoiseEvaluator<Vec>, Vec: VecType<f64>>(
    delta: Vec,
    origin: Vec,
    grid: Vec,
    perm: &PermTable,
) -> f64 {
    let shifted: Vec = origin - delta - NoiseEvaluatorType::SQUISH_POINT * delta.sum();
    let attn: f64 = 2.0 - shifted.get_attenuation_factor();
    if attn > 0.0 {
        let attn2 = attn * attn;
        return attn2 * attn2 * NoiseEvaluatorType::extrapolate(grid + delta, shifted, perm);
    }

    0.0
}

pub fn floor(x: f64) -> i64 {
    x.floor() as i64
}

pub fn to_f64(x: i64) -> f64 {
    x as f64
}
