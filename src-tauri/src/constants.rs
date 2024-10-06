use std::f32::consts::PI;

use rustfft::num_complex::Complex;

use crate::types::IIR2;

pub const NUM_FILTERS: usize = 5;

#[tauri::command]
pub fn get_num_filters() -> usize {
    NUM_FILTERS
}

pub const TEST_FILE: &str = "reisman.wav";
pub const ASSETS_PATH: &str = "assets";

pub const SAMPLING_RATE: f32 = 44100.0;
pub const NYQUIST: f32 = SAMPLING_RATE / 2.0;
pub const DOWN_RATE: usize = 1;

pub const CZERO: Complex<f32> = Complex { re: 0.0, im: 0.0 };
pub fn czerov(n: usize) -> Vec<Complex<f32>> {
    vec![Complex { re: 0.0, im: 0.0 }; n]
}

pub fn from_log(g: f32) -> f32 {
    (10.0_f32).powf(g / 20.0)
}

pub fn biquad(gain: f32, freq: f32, Q: f32) -> IIR2 {
    let A = (gain / 40.0).powf(10.0);
    let w0 = (2.0 * PI * freq) / SAMPLING_RATE;
    let alpha = (w0).sin() / 2.0 / Q;
    IIR2 {
        b0: 1.0 + alpha * A,
        b1: -2.0 * w0.cos(),
        b2: 1.0 - alpha * A,
        a0: 1.0 + alpha / A,
        a1: -2.0 * w0.cos(),
        a2: 1.0 - alpha / A,
        x: [0.0, 0.0],
        y: [0.0, 0.0],
    }
}
