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

pub const SAMPLING_RATE: f32 = 44100.0;

pub fn from_log(g: f32) -> f32 {
    (10.0_f32).powf(g / 20.0)
}
