fn get_quorem(a: u64, b: u64) -> (u64, u64) {
    (a % b, (a - a % b) / b)
}

pub fn get_hms(seconds: u64) -> (u64, u64, u64) {
    let (s, total_m) = get_quorem(seconds, 60);
    let (m, h) = get_quorem(total_m, 60);
    (h, m, s)
}
