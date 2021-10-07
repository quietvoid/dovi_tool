pub const ST2084_Y_MAX: f64 = 10000.0;
pub const ST2084_M1: f64 = 2610.0 / 16384.0;
pub const ST2084_M2: f64 = (2523.0 / 4096.0) * 128.0;
pub const ST2084_C1: f64 = 3424.0 / 4096.0;
pub const ST2084_C2: f64 = (2413.0 / 4096.0) * 32.0;
pub const ST2084_C3: f64 = (2392.0 / 4096.0) * 32.0;

#[inline(always)]
pub fn nits_to_pq(nits: u32) -> f64 {
    let y = nits as f64 / ST2084_Y_MAX;

    ((ST2084_C1 + ST2084_C2 * y.powf(ST2084_M1)) / (1.0 + ST2084_C3 * y.powf(ST2084_M1)))
        .powf(ST2084_M2)
}
