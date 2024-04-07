#![allow(dead_code, unused_imports)]
use dasp_ring_buffer as ring_buf;
use rand::Rng;

use rustfft::num_complex::{Complex32, ComplexFloat};
use rustfft::{num_complex::Complex, FftPlanner};
use std::f32::consts::PI;
// use std::simd::f32x4;
use std::time::Instant;

use crate::constants::CZERO;

#[derive(Clone, Debug)]
pub struct SDFT {
    pub size: usize,
    pub time_history: ring_buf::Fixed<Vec<Complex<f32>>>,
    pub freq_history: Vec<Complex<f32>>,
    pub smooth_noise_history: Vec<Complex<f32>>,
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
            smooth_noise_history: freq_history.clone(),
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
        smooth_gain: f32,
    ) -> f32 {
        let oldest_input = self.time_history.get(0);
        let delta = signal - oldest_input;
        let mut out;
        let mut denoise;
        let mut smoothed_noise = CZERO;
        let mut arg;
        let mut mag;
        self.inv_time = CZERO;

        for freq in 0..self.size {
            // get spectrum of input
            self.new_freq[freq] = (self.freq_history[freq] + delta) * self.fkernel[freq];
            mag = self.new_freq[freq].norm();
            arg = self.new_freq[freq].arg();
            //pre process the magnitude spectrum? filter out variations...do they mean remove from the spectrum as if it was a time signal? or just remove high frequencies in the spectrum

            // arg += rand::thread_rng().gen_range(0..1000000) as f32 / 1000000.0 - 0.5;

            out = mag - noise_gain * noise_spectrum[freq];
            denoise = Complex32::from_polar(out.clamp(0.0, f32::MAX), arg);
            // if out < 0.0 {
            //     denoise *= -1.0;
            // }
            // might need to do some post processing, it is nonlinear...what about upsampling? upsample the original file, process that
            smoothed_noise =
                smooth_gain * self.smooth_noise_history[freq] + (1.0 - smooth_gain) * denoise;
            self.smooth_noise_history[freq] = smoothed_noise;

            // inverse
            self.inv_time += smoothed_noise * self.ikernel[freq];
        }
        self.freq_history = self.new_freq.clone();
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
