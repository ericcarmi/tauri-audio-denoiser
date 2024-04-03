#![allow(dead_code, unused_imports)]
use dasp_ring_buffer as ring_buf;
use rustfft::{num_complex::Complex, FftPlanner};
use std::f32::consts::PI;
// use std::simd::f32x4;
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct SDFT {
    pub size: usize,
    pub time_history: ring_buf::Fixed<Vec<Complex<f32>>>,
    pub freq_history: Vec<Complex<f32>>,
    pub inv_time: Complex<f32>,
    pub new_freq: Vec<Complex<f32>>,
    pub fkernel: Vec<Complex<f32>>,
    pub ikernel: Vec<Complex<f32>>,
    pub time_output: Complex<f32>,
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
        let mut new_freq = vec![
            Complex {
                re: 0.0f32,
                im: 0.0f32
            };
            size
        ];
        let mut inv_time = Complex {
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

        Self {
            size,
            time_history,
            freq_history: freq_history.clone(),
            new_freq: freq_history,
            inv_time,
            fkernel,
            ikernel,
            time_output: Complex { re: 0.0, im: 0.0 },
        }
    }

    pub fn process(&mut self, signal: f32) -> f32 {
        // simd eventually this loop is slow as shit
        // inverse should also be simd eventually
        let oldest_input = self.time_history.get(0);
        let delta = signal - oldest_input;
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
    // this is wrong, should return time vec
    pub fn process_vec(&mut self, signal: &Vec<f32>) -> f32 {
        for t in 0..signal.len() {
            let oldest_input = self.time_history.get(self.size - 1).clone();
            for freq in 0..self.size {
                self.new_freq[freq] =
                    (self.freq_history[freq] + signal[t] - oldest_input) * self.fkernel[freq];
                self.freq_history[freq] = self.new_freq[freq];
            }
            for freq in 0..self.size {
                self.time_output += self.new_freq[freq] * self.ikernel[freq];
            }

            self.time_history.push(Complex {
                re: signal[t],
                im: 0.0,
            });
            self.freq_history[t] = Complex {
                re: signal[t],
                im: 0.0,
            };
        }

        return self.time_output.re;
    }

    pub fn norm_vec(&self) -> Vec<f32> {
        self.new_freq.iter().map(|x| x.norm()).collect()
    }
}

impl Default for SDFT {
    fn default() -> Self {
        SDFT::new(256)
    }
}
