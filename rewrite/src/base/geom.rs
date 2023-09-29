// Transform
// from spherical coordinate system (r, theta, phi)
// to Cartesian coordinate system (x, y, z)
//
// x=rsinϕcosθ
// y=rsinϕsinθ
// z=rcosϕ
//
// https://keisan.casio.com/exec/system/1359534351
pub fn to_cartesian(v: (f64, f64, f64)) -> (f64, f64, f64) {
    fn to_rad(deg: f64) -> f64 {
        deg * std::f64::consts::PI / 180.0
    }
    let r = v.0;
    let th = to_rad(v.1);
    let ph = to_rad(v.2);
    let x = r * ph.sin() * th.cos();
    let y = r * ph.sin() * th.sin();
    let z = r * ph.cos();
    (x, y, z)
}
