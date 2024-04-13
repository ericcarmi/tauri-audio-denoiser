use cpal::Stream;
use dasp_ring_buffer::Fixed;
use rustfft::num_complex::{Complex, Complex32};
use serde::{Deserialize, Serialize};
use std::{f32::consts::PI, sync::Mutex};

use crate::{
    constants::{czerov, CZERO},
    sdft::SDFT,
};

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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum StereoControl {
    Left,
    Right,
    Both,
}

/// a message from a single channel (left or right)
#[derive(Clone, Debug, Copy)]
pub struct ChannelMessage {
    pub time: Option<f32>,
    pub clean: Option<bool>,
    pub bp1: Option<IIR2>,
    pub bp2: Option<IIR2>,
    pub bp3: Option<IIR2>,
    pub bp4: Option<IIR2>,
    pub bp5: Option<IIR2>,
    pub output_gain: Option<f32>,
    pub noise_gain: Option<f32>,
    pub pre_smooth_gain: Option<f32>,
    pub post_smooth_gain: Option<f32>,
    pub id: &'static str,
}

impl Default for ChannelMessage {
    fn default() -> Self {
        Self {
            time: None,
            clean: None,
            bp1: None,
            bp2: None,
            bp3: None,
            bp4: None,
            bp5: None,
            output_gain: None,
            noise_gain: None,
            pre_smooth_gain: None,
            post_smooth_gain: None,
            id: "left",
        }
    }
}

// not sure if this is needed, maybe it should be in AudioParams?
// impl ChannelMessage {
//     pub fn filters_as_slice(&self) -> [IIR2; 5] {
//         return [self.bp1, self.bp2, self.bp3, self.bp4, self.bp5];
//     }
// }

// use options with everything...a little annoying but then use None when passing to ignore most sub-structs
// this struct is for messages sent from UI to audio thread
#[derive(Clone, Debug)]
pub struct Message {
    pub left_channel: Option<ChannelMessage>,
    pub right_channel: Option<ChannelMessage>,
    pub file_path: Option<String>,
    pub stereo_control: StereoControl,
}

// use all None for default message to shorten other functions that send one thing at a time
impl Default for Message {
    fn default() -> Self {
        Self {
            file_path: None,
            stereo_control: StereoControl::Both,
            left_channel: Some(ChannelMessage::default()),
            right_channel: Some(ChannelMessage {
                id: "right",
                ..Default::default()
            }),
        }
    }
}

impl Message {
    pub fn receive(&self, params: &mut StereoAudioParams) {
        if let Some(ch) = self.left_channel {
            // would eventually like to iterate over all filters in the filter bank instead of doing this
            if let Some(bp) = ch.bp1 {
                params.left.filter_bank.bp1.update_coeffs(bp);
                params.left.noise_spectrum = params
                    .left
                    .filter_bank
                    .parallel_transfer(params.left.dft_size);
            }
            if let Some(bp) = ch.bp2 {
                params.left.filter_bank.bp2.update_coeffs(bp);
                params.left.noise_spectrum = params
                    .left
                    .filter_bank
                    .parallel_transfer(params.left.dft_size);
            }
            if let Some(bp) = ch.bp3 {
                params.left.filter_bank.bp3.update_coeffs(bp);
                params.left.noise_spectrum = params
                    .left
                    .filter_bank
                    .parallel_transfer(params.left.dft_size);
            }
            if let Some(bp) = ch.bp4 {
                params.left.filter_bank.bp5.update_coeffs(bp);
                params.left.noise_spectrum = params
                    .left
                    .filter_bank
                    .parallel_transfer(params.left.dft_size);
            }
            if let Some(bp) = ch.bp5 {
                params.left.filter_bank.bp5.update_coeffs(bp);
                params.left.noise_spectrum = params
                    .left
                    .filter_bank
                    .parallel_transfer(params.left.dft_size);
            }
            if let Some(t) = ch.time {
                params.left.time = (params.num_file_samples as f32 * t) as usize;
                params.left.sdft.freq_history = czerov(params.left.dft_size);
                params.left.sdft.time_history =
                    Fixed::from(vec![Complex::new(0.0, 0.0); params.left.dft_size]);
            }
            if let Some(c) = ch.clean {
                params.left.clean = c;
                params.left.sdft.freq_history = czerov(params.left.dft_size);
                params.left.sdft.time_history =
                    Fixed::from(vec![Complex::new(0.0, 0.0); params.left.dft_size]);
            }
            if let Some(g) = ch.output_gain {
                params.left.output_gain = g;
            }
            if let Some(g) = ch.noise_gain {
                params.left.noise_gain = g;
            }
            if let Some(g) = ch.pre_smooth_gain {
                params.left.pre_smooth_gain = g;
            }
            if let Some(g) = ch.post_smooth_gain {
                params.left.post_smooth_gain = g;
            }
        }
        if let Some(ch) = self.right_channel {
            self.recv_channel(ch, params);
        }
    }
    pub fn recv_channel(&self, channel: ChannelMessage, params: &mut StereoAudioParams) {}
}

#[derive(Clone)]
pub struct AudioParams {
    pub time: usize,
    pub clean: bool,
    pub bp1: IIR2,
    pub bp2: IIR2,
    pub bp3: IIR2,
    pub bp4: IIR2,
    pub bp5: IIR2,
    pub filter_bank: FilterBank,
    pub bypass: Vec<bool>,
    pub output_gain: f32,
    pub noise_gain: f32,
    pub pre_smooth_gain: f32,
    pub post_smooth_gain: f32,
    pub dft_size: usize,
    pub output_spectrum: Vec<f32>,
    pub noise_spectrum: Vec<f32>,
    pub sdft: SDFT,
}

// bp1..5 should be changed to filterbank, so new filters can be added easily

impl AudioParams {
    pub fn new() -> Self {
        let fb = FilterBank::new();
        Self {
            dft_size: 256,
            time: 0,
            clean: false,
            bp1: fb.bp1,
            bp2: fb.bp2,
            bp3: fb.bp3,
            bp4: fb.bp4,
            bp5: fb.bp5,
            bypass: vec![],
            output_gain: 1.0,
            noise_gain: 0.0,
            pre_smooth_gain: 0.5,
            post_smooth_gain: 0.5,
            output_spectrum: vec![],
            noise_spectrum: vec![],
            filter_bank: fb,
            sdft: SDFT::new(256),
        }
    }
    // pub fn filter_bank(&self) -> [IIR2; 5] {
    //     [self.bp1, self.bp2, self.bp3, self.bp4, self.bp5]
    // }
}

pub struct StereoAudioParams {
    pub left: AudioParams,
    pub right: AudioParams,
    pub stereo_control: StereoControl,
    pub clean: bool,
    pub num_file_samples: usize,
    pub file_path: String,
    pub is_stereo: bool,
}

impl StereoAudioParams {
    pub fn new() -> Self {
        Self {
            left: AudioParams::new(),
            right: AudioParams::new(),
            stereo_control: StereoControl::Both,
            clean: false,
            num_file_samples: 0,
            file_path: "".to_string(),
            is_stereo: false,
        }
    }
}
