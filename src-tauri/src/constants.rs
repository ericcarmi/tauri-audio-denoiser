use rustfft::num_complex::Complex;

// pub const TEST_FILE_PATH: &str = "assets/chirp.wav";
// pub const TEST_FILE_PATH: &str = "assets/440-7040-whitenoise.wav";
// pub const TEST_FILE_PATH: &str = "assets/440-whitenoise.wav";
pub const TEST_FILE_PATH: &str = "assets/reisman.wav";
pub const ASSETS_PATH: &str = "assets/";

pub const CZERO: Complex<f32> = Complex { re: 0.0, im: 0.0 };
pub fn czerov(n: usize) -> Vec<Complex<f32>> {
    vec![Complex { re: 0.0, im: 0.0 }; n]
}

pub const REDIS_PORT: &str = "6380";
