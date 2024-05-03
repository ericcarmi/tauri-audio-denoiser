use cpal::Stream;
use rusqlite::types::FromSql;
use rustfft::num_complex::Complex32;
use serde::{Deserialize, Serialize};
use std::{f32::consts::PI, sync::Mutex};
use ts_rs::TS;

use crate::{
    constants::{CZERO, SAMPLING_RATE},
    messages::{AudioUIMessage, Message},
    sdft::SDFT,
};

// cpal stream
pub struct MStream(pub Mutex<Stream>);
// receive message for ui
pub struct MUIReceiver(pub Mutex<tauri::async_runtime::Receiver<AudioUIMessage>>);
// send message from audio to ui
pub struct MAudioSender(pub Mutex<tauri::async_runtime::Sender<AudioUIMessage>>);
// send message from ui to audio thread
pub struct MSender(pub Mutex<tauri::async_runtime::Sender<Message>>);
pub struct MStreamSend(pub Mutex<StreamSend>);

pub struct StreamSend {
    pub stream: MStream,
    pub msender: MSender,
    pub mreceiver: MUIReceiver,
    pub mtx_ui: MAudioSender,
}
unsafe impl Sync for MStream {}
unsafe impl Send for MStream {}
unsafe impl Send for MSender {}
unsafe impl Sync for MSender {}
unsafe impl Send for MUIReceiver {}
unsafe impl Sync for MUIReceiver {}
unsafe impl Send for MAudioSender {}
unsafe impl Sync for MAudioSender {}
unsafe impl Send for MStreamSend {}
unsafe impl Sync for MStreamSend {}
unsafe impl Send for StreamSend {}
unsafe impl Sync for StreamSend {}
unsafe impl Send for AudioUIMessage {}
unsafe impl Sync for AudioUIMessage {}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, TS)]
#[ts(export)]
pub struct BPF {
    pub gain: f32,
    pub freq: f32,
    pub Q: f32,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, TS)]
#[ts(export)]
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

impl From<BPF> for IIR2 {
    fn from(bpf: BPF) -> Self {
        let A = (bpf.gain / 40.0).powf(10.0);
        let w0 = (2.0 * PI * bpf.freq) / SAMPLING_RATE;
        let alpha = (w0).sin() / 2.0 / bpf.Q;
        Self {
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

#[derive(Clone, Serialize, Deserialize, Debug, TS)]
#[ts(export)]
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
    // pub fn process(&self, data: f32) -> f32 {
    //     0.0
    // }

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
        for (_i, filt) in self.as_slice().iter().enumerate() {
            let h = filt.freq_response(n);

            H.iter_mut().enumerate().for_each(|(i, x)| *x += h[i] / l);
            // for j in H.clone() {
            // }
        }
        // take norm after summing filters

        let mut out: Vec<f32> = H.iter().map(|x| x.norm()).collect();
        if out[0].is_nan() {
            out[0] = 0.0;
        }
        out
    }
}

// maybe rename to StereoChoice
#[derive(Clone, Debug, Deserialize, Serialize, Copy, TS)]
#[ts(export)]
pub enum StereoControl {
    Left = 0,
    Right = 1,
    Both = 2,
}

impl FromSql for StereoControl {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        use StereoControl::*;
        match value.as_str().unwrap() {
            "left" => Ok(Left),
            "right" => Ok(Right),
            "both" => Ok(Both),
            _ => Err(rusqlite::types::FromSqlError::InvalidType),
        }
    }
}

impl StereoControl {
    pub fn as_str(&self) -> &str {
        use StereoControl::*;
        match self {
            Left => "Left",
            Right => "Right",
            Both => "Both",
        }
    }
    pub fn iter() -> [StereoControl; 3] {
        use StereoControl::*;
        [Left, Right, Both]
    }
    pub fn is_left(&self) -> bool {
        if self.as_str() == "Left" {
            return true;
        }
        false
    }
    pub fn is_right(&self) -> bool {
        if self.as_str() == "Right" {
            return true;
        }
        false
    }
    pub fn is_both(&self) -> bool {
        if self.as_str() == "Both" {
            return true;
        }
        false
    }
}

/// audio params to be used in the audio thread -- some variables can be set directly from messages, others are computed (spectra, sdft) and can skip being serialized
// rename to channel params?
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AudioParams {
    pub time: usize,
    pub clean: bool,
    pub mute: bool,
    pub dft_size: usize,
    pub bp1: IIR2,
    pub bp2: IIR2,
    pub bp3: IIR2,
    pub bp4: IIR2,
    pub bp5: IIR2,
    pub filter_bank: FilterBank,
    pub output_gain: f32,
    pub noise_gain: f32,
    pub pre_smooth_gain: f32,
    pub post_smooth_gain: f32,
    #[serde(skip)]
    pub output_spectrum: Vec<f32>,
    #[serde(skip)]
    pub noise_spectrum: Vec<f32>,
    #[serde(skip)]
    pub sdft: SDFT,
}

impl AudioParams {
    pub fn new() -> Self {
        let fb = FilterBank::new();
        let n = 256;
        Self {
            dft_size: n,
            mute: false,
            time: 0,
            clean: false,
            bp1: fb.bp1,
            bp2: fb.bp2,
            bp3: fb.bp3,
            bp4: fb.bp4,
            bp5: fb.bp5,
            output_gain: 1.0,
            noise_gain: 0.0,
            pre_smooth_gain: 0.5,
            post_smooth_gain: 0.5,
            output_spectrum: vec![],
            noise_spectrum: fb.parallel_transfer(n),
            filter_bank: fb,
            sdft: SDFT::new(n),
        }
    }
    // pub fn filter_bank(&self) -> [IIR2; 5] {
    //     [self.bp1, self.bp2, self.bp3, self.bp4, self.bp5]
    // }
    // pub fn set_filters(&mut self, filters: Vec<IIR2>) {
    //     let old = self.filter_bank();
    //     for (o, n) in old.iter().zip(filters) {
    //         o = n;
    //     }
    // }
}

impl Default for AudioParams {
    fn default() -> Self {
        let fb = FilterBank::new();
        let n = 256;
        Self {
            dft_size: n,
            mute: false,
            time: 0,
            clean: false,
            bp1: fb.bp1,
            bp2: fb.bp2,
            bp3: fb.bp3,
            bp4: fb.bp4,
            bp5: fb.bp5,
            output_gain: 1.0,
            noise_gain: 0.0,
            pre_smooth_gain: 0.5,
            post_smooth_gain: 0.5,
            output_spectrum: vec![],
            noise_spectrum: fb.parallel_transfer(n),
            filter_bank: fb,
            sdft: SDFT::new(n),
        }
    }
}

// just copy of audio params? ehhhh
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ChannelParams {
    pub time: usize,
    pub clean: bool,
    pub mute: bool,
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
}

/// stereo params includes AudioParams for both channels as well as other params that are independent of the channels
#[derive(Debug, TS, Serialize, Deserialize)]
#[ts(export)]
pub struct StereoAudioParams {
    pub id: i64,
    pub left: AudioParams,
    pub right: AudioParams,
    pub stereo_control: StereoControl,
    pub clean: bool,
    pub num_file_samples: usize,
    pub file_path: String,
    pub is_stereo: bool,
    pub time: usize,
}

impl StereoAudioParams {
    pub fn new() -> Self {
        Self {
            id: 0,
            left: AudioParams::new(),
            right: AudioParams::new(),
            stereo_control: StereoControl::Both,
            clean: false,
            num_file_samples: 0,
            file_path: "".to_string(),
            is_stereo: false,
            time: 0,
        }
    }
}

impl Default for StereoAudioParams {
    fn default() -> Self {
        Self {
            id: 0,
            left: AudioParams::new(),
            right: AudioParams::new(),
            stereo_control: StereoControl::Both,
            clean: false,
            num_file_samples: 0,
            file_path: "".to_string(),
            is_stereo: false,
            time: 0,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UIParams {
    pub bpfs: Vec<BPF>,
    pub clean: bool,
    pub output_gain: f32,
    pub noise_gain: f32,
    pub pre_smooth_gain: f32,
    pub post_smooth_gain: f32,
    pub left_mute: bool,
    pub right_mute: bool,
}

impl UIParams {
    pub fn new() -> Self {
        Self {
            clean: false,
            output_gain: 1.0,
            noise_gain: 0.0,
            pre_smooth_gain: 0.5,
            post_smooth_gain: 0.5,
            bpfs: vec![],
            left_mute: false,
            right_mute: false,
        }
    }
}
