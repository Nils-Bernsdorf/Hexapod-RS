#[inline]
pub fn clamp_abs(val: f64, max: f64) -> f64 {
    if val.abs() < max {
        val
    } else {
        val.signum() * max
    }
}