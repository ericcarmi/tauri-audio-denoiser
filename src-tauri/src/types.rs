use cpal::Stream;
use rustfft::num_complex::Complex32;
use serde::{Deserialize, Serialize};
use std::{f32::consts::PI, sync::Mutex};

use crate::constants::CZERO;

pub struct MStream(pub Mutex<Stream>);

unsafe impl Sync for MStream {}
unsafe impl Send for MStream {}

pub struct MUIReceiver(pub Mutex<tauri::async_runtime::Receiver<Vec<f32>>>);
pub struct MSender(pub Mutex<tauri::async_runtime::Sender<Message>>);

pub struct MStreamSend(pub Mutex<StreamSend>);

pub struct StreamSend {
    pub stream: MStream,
    pub msender: MSender,
    pub mreceiver: MUIReceiver,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct Bpf {
    pub gain: f32,
    pub freq: f32,
    pub Q: f32,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct IIR2 {
    pub b0: f32,
    pub b1: f32,
    pub b2: f32,
    pub a0: f32,
    pub a1: f32,
    pub a2: f32,
    pub x: [f32; 2],
    pub y: [f32; 2],
}

impl IIR2 {
    pub fn new() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a0: 1.0,
            a1: 0.0,
            a2: 0.0,
            x: [0.0, 0.0],
            y: [0.0, 0.0],
        }
    }
    pub fn process(&mut self, data: f32) -> f32 {
        let out = (self.b0 * data + self.b1 * self.x[0] + self.b2 * self.x[1]
            - self.a1 * self.y[0]
            - self.a2 * self.y[1])
            / self.a0;
        self.x[1] = self.x[0];
        self.x[0] = data;
        self.y[1] = self.y[0];
        self.y[0] = out;

        out
    }

    pub fn update_coeffs(&mut self, iir: IIR2) {
        self.b0 = iir.b0;
        self.b1 = iir.b1;
        self.b2 = iir.b2;
        self.a0 = iir.a0;
        self.a1 = iir.a1;
        self.a2 = iir.a2;
    }
    pub fn freq_response(&self, n: usize) -> Vec<Complex32> {
        let mut H = vec![];
        let L = n as f32;
        for i in 0..n {
            let x = (-PI * i as f32 / L).cos();
            let y = (-PI * i as f32 / L).sin();
            let z = Complex32 { re: x, im: y };
            let z2 = z * z;

            let w = (self.b0 + self.b1 * z + self.b2 * z2) / (self.a0 + self.a1 * z + self.a2 * z2);

            H.push(w);
        }
        H
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FilterBank {
    pub bp1: IIR2,
    pub bp2: IIR2,
    pub bp3: IIR2,
    pub bp4: IIR2,
    pub bp5: IIR2,
}

impl FilterBank {
    pub fn new() -> Self {
        let bp = IIR2::new();
        Self {
            bp1: bp,
            bp2: bp,
            bp3: bp,
            bp4: bp,
            bp5: bp,
        }
    }
    // function to process filter bank in parallel
    pub fn process(&self, data: f32) -> f32 {
        0.0
    }

    /// to iterate over all the filters
    pub fn as_slice(&self) -> [IIR2; 5] {
        [self.bp1, self.bp2, self.bp3, self.bp4, self.bp5]
    }

    pub fn parallel_transfer(&self, n: usize) -> Vec<f32> {
        let mut H: Vec<Complex32> = vec![CZERO; n];
        let l = Complex32 {
            re: self.as_slice().len() as f32,
            im: 0.0,
        };

        // loop over all filters first
        for (i, filt) in self.as_slice().iter().enumerate() {
            let h = filt.freq_response(n);
            // println!("{}\n {:?}", i, h);

            H.iter_mut().enumerate().for_each(|(i, x)| *x += h[i] / l);
            // for j in H.clone() {
            //     println!("{:?}", j);
            // }
        }
        // take norm after summing filters
        // println!("{:?}", H);

        let mut out: Vec<f32> = H.iter().map(|x| x.norm()).collect();
        if out[0].is_nan() {
            out[0] = 0.0;
        }
        out
    }
}

// use options with everything...a little annoying but then use None when passing to ignore most sub-structs
// this struct is for messages sent from UI to audio thread
#[derive(Clone)]
pub struct Message {
    pub time: Option<f32>,
    pub clean: Option<bool>,
    pub bp1: Option<IIR2>,
    pub bp2: Option<IIR2>,
    pub bp3: Option<IIR2>,
    pub bp4: Option<IIR2>,
    pub bp5: Option<IIR2>,
    pub bypass: Option<Vec<Option<bool>>>,
    pub output_gain: Option<f32>,
    pub noise_gain: Option<f32>,
    pub pre_smooth_gain: Option<f32>,
    pub post_smooth_gain: Option<f32>,
    pub noise_variance: Option<f32>,
    pub file_path: Option<String>,
}

// use all None for default message to shorten other functions that send one thing at a time
impl Default for Message {
    fn default() -> Self {
        Self {
            time: None,
            clean: None,
            bp1: None,
            bp2: None,
            bp3: None,
            bp4: None,
            bp5: None,
            bypass: None,
            output_gain: None,
            noise_gain: None,
            pre_smooth_gain: None,
            noise_variance: None,
            post_smooth_gain: None,
            file_path: None,
        }
    }
}
