#![allow(dead_code, unused_imports)]
use dasp_ring_buffer as ring_buf;
use rand::Rng;

use rustfft::num_complex::{Complex32, ComplexFloat};
use rustfft::{num_complex::Complex, FftPlanner};
use std::f32::consts::PI;
// use std::simd::f32x4;
use std::time::Instant;

use crate::constants::{czerov, CZERO};

#[derive(Clone, Debug)]
pub struct SDFT {
    pub size: usize,
    pub time_history: ring_buf::Fixed<Vec<Complex<f32>>>,
    pub freq_history: Vec<Complex<f32>>,
    pub pre_smooth_noise_history: Vec<Complex<f32>>,
    pub post_smooth_noise_history: Vec<Complex<f32>>,
    pub inv_time: Complex<f32>,
    pub new_freq: Vec<Complex<f32>>,
    pub fkernel: Vec<Complex<f32>>,
    pub ikernel: Vec<Complex<f32>>,
    pub time_output: Complex<f32>,
    pub filter: Vec<f32>,
}

impl SDFT {
    pub fn new(size: usize) -> Self {
        let mut fkernel = vec![
            Complex {
                re: 0.0f32,
                im: 0.0f32
            };
            size
        ];
        let mut ikernel = vec![
            Complex {
                re: 0.0f32,
                im: 0.0f32
            };
            size
        ];
        let freq_history = vec![
            Complex {
                re: 0.0f32,
                im: 0.0f32
            };
            size
        ];
        let new_freq = vec![
            Complex {
                re: 0.0f32,
                im: 0.0f32
            };
            size
        ];
        let inv_time = Complex {
            re: 0.0f32,
            im: 0.0f32,
        };
        // let mut time_history = AllocRingBuffer::new(size);
        let mut time_history = ring_buf::Fixed::from(vec![Complex::new(0.0, 0.0); size]);

        for i in 0..size {
            fkernel[i] = Complex::new(0.0, -2.0 * PI * i as f32 / size as f32).exp();

            ikernel[i] = Complex::new(0.0, 2.0 * PI * i as f32 / size as f32).exp() / size as f32;
            time_history.push(Complex::new(0.0, 0.0));
        }
        let filter: Vec<f32> = vec![0.0; size];

        Self {
            size,
            time_history,
            freq_history: freq_history.clone(),
            pre_smooth_noise_history: czerov(size),
            post_smooth_noise_history: freq_history.clone(),
            new_freq,
            inv_time,
            fkernel,
            ikernel,
            time_output: Complex { re: 0.0, im: 0.0 },
            filter,
        }
    }

    pub fn process(&mut self, signal: f32) -> f32 {
        // simd eventually this loop is slow as shit
        // inverse should also be simd eventually
        let oldest_input = self.time_history.get(0);
        let delta = signal - oldest_input;
        self.inv_time = CZERO;
        for freq in 0..self.size {
            self.new_freq[freq] = (self.freq_history[freq] + delta) * self.fkernel[freq];
            self.inv_time += self.new_freq[freq] * self.ikernel[freq];
        }
        self.freq_history = self.new_freq.clone();
        self.time_history.push(Complex {
            re: signal,
            im: 0.0,
        });
        return self.inv_time.re;
    }

    pub fn spectral_subtraction(
        &mut self,
        signal: f32,
        noise_spectrum: &Vec<f32>,
        noise_gain: f32,
        pre_smooth_gain: f32,
        post_smooth_gain: f32,
        noise_variance: f32,
    ) -> f32 {
        let oldest_input = self.time_history.get(0);
        let delta = signal - oldest_input;
        let mut out: f32;
        let mut denoise;
        let mut post_smoothed_noise;
        let mut pre_smoothed_noise;
        let mut arg;
        let mut mag;
        let mut noise;
        self.inv_time = CZERO;

        for freq in 0..self.size {
            // get spectrum of input
            self.new_freq[freq] = (self.freq_history[freq] + delta) * self.fkernel[freq];

            noise = (noise_spectrum[freq] - 1.0).abs();
            // smooth the noise variance
            pre_smoothed_noise = pre_smooth_gain * self.pre_smooth_noise_history[freq]
                + (1.0 - pre_smooth_gain) * self.new_freq[freq];

            // polar
            mag = pre_smoothed_noise.norm();
            arg = pre_smoothed_noise.arg();
            out = mag - noise_gain * noise;
            denoise = Complex32::from_polar(out.clamp(1e-6, f32::MAX), arg);

            // more smoothing
            post_smoothed_noise = post_smooth_gain * self.post_smooth_noise_history[freq]
                + (1.0 - post_smooth_gain) * denoise;
            //delay
            self.freq_history[freq] = self.new_freq[freq];
            self.pre_smooth_noise_history[freq] = pre_smoothed_noise;
            self.post_smooth_noise_history[freq] = post_smoothed_noise;

            // inverse
            self.inv_time += post_smoothed_noise * self.ikernel[freq];
        }
        // self.freq_history = self.new_freq.clone();
        self.time_history.push(Complex {
            re: signal,
            im: 0.0,
        });

        self.inv_time.re
    }

    /// magnitude of frequency spectrum
    pub fn norm_vec(&self) -> Vec<f32> {
        self.new_freq.iter().map(|x| x.norm()).collect()
    }
    pub fn phase_vec(&self) -> Vec<f32> {
        self.new_freq.iter().map(|x| x.arg()).collect()
    }
    pub fn norm(&self, n: usize) -> f32 {
        self.new_freq[n].norm()
    }
    pub fn phase(&self, n: usize) -> f32 {
        self.new_freq[n].arg()
    }
}

impl Default for SDFT {
    fn default() -> Self {
        SDFT::new(256)
    }
}
