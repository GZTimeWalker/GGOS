extern crate libm;

macro_rules! no_mangle {
    ($(fn $fun:ident($($iid:ident : $ity:ty),+) -> $oty:ty;)+) => {
        $(
            #[no_mangle]
            pub extern "C" fn $fun($($iid: $ity),+) -> $oty {
                self::libm::$fun($($iid),+)
            }
        )+
    }
}

no_mangle! {
    fn acos(x: f64) -> f64;
    fn asin(x: f64) -> f64;
    fn cbrt(x: f64) -> f64;
    fn expm1(x: f64) -> f64;
    fn hypot(x: f64, y: f64) -> f64;
    fn tan(x: f64) -> f64;
    fn cos(x: f64) -> f64;
    fn expf(x: f32) -> f32;
    fn log2(x: f64) -> f64;
    fn log2f(x: f32) -> f32;
    fn log10(x: f64) -> f64;
    fn log10f(x: f32) -> f32;
    fn log(x: f64) -> f64;
    fn logf(x: f32) -> f32;
    fn fmin(x: f64, y: f64) -> f64;
    fn fminf(x: f32, y: f32) -> f32;
    fn fmax(x: f64, y: f64) -> f64;
    fn fmaxf(x: f32, y: f32) -> f32;
    fn round(x: f64) -> f64;
    fn roundf(x: f32) -> f32;
    fn sin(x: f64) -> f64;
    fn pow(x: f64, y: f64) -> f64;
    fn powf(x: f32, y: f32) -> f32;
    fn fmod(x: f64, y: f64) -> f64;
    fn fmodf(x: f32, y: f32) -> f32;
    fn acosf(n: f32) -> f32;
    fn atan2f(a: f32, b: f32) -> f32;
    fn atanf(n: f32) -> f32;
    fn coshf(n: f32) -> f32;
    fn expm1f(n: f32) -> f32;
    fn fdim(a: f64, b: f64) -> f64;
    fn fdimf(a: f32, b: f32) -> f32;
    fn log1pf(n: f32) -> f32;
    fn sinhf(n: f32) -> f32;
    fn tanhf(n: f32) -> f32;
    fn ldexp(f: f64, n: i32) -> f64;
    fn ldexpf(f: f32, n: i32) -> f32;
}
