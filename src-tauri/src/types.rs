use cpal::Stream;
use rusqlite::types::FromSql;
use rustfft::num_complex::Complex32;
use serde::{Deserialize, Serialize};
use std::{f32::consts::PI, sync::Mutex};
use ts_rs::TS;

use crate::{
    constants::{CZERO, NUM_FILTERS, SAMPLING_RATE},
    messages::{AudioUIMessage, UIAudioMessage},
    sdft::SDFT,
};

// cpal stream
pub struct MStream(pub Mutex<Stream>);
// receive message for ui
pub struct MUIReceiver(pub Mutex<tauri::async_runtime::Receiver<AudioUIMessage>>);
// send message from audio to ui
pub struct MAudioSender(pub Mutex<tauri::async_runtime::Sender<AudioUIMessage>>);
// send message from ui to audio thread
pub struct MSender(pub Mutex<tauri::async_runtime::Sender<UIAudioMessage>>);
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
struct Complex {
    re: f32,
    im: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize, Copy, TS)]
#[ts(export)]
pub enum PlotScale {
    Linear,
    Mel,
    Log,
    Bark,
}

/// user-facing params that control a bandpass filter, convert to IIR for internal audio processing
#[derive(Clone, Copy, Serialize, Deserialize, Debug, TS)]
#[ts(export)]
pub struct BPF {
    pub gain: f32,
    pub freq: f32,
    pub Q: f32,
}

impl BPF {
    pub fn new() -> Self {
        Self {
            gain: 0.0,
            freq: 1000.0,
            Q: 1.0,
        }
    }
}

/// IIR filter, second order
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

impl From<UIFilters> for Filters {
    fn from(fb: UIFilters) -> Self {
        let mut bank = [IIR2::new(); NUM_FILTERS];
        for i in 0..NUM_FILTERS {
            bank[i] = IIR2::from(fb.bank[i]);
        }
        Self { bank }
    }
}

#[derive(Clone, Serialize, Copy, Deserialize, Debug, TS)]
#[ts(export)]
pub struct Filters {
    pub bank: [IIR2; NUM_FILTERS],
}
impl Default for Filters {
    fn default() -> Self {
        let bank = [IIR2::new(); NUM_FILTERS];
        Self { bank }
    }
}
impl Filters {
    pub fn new() -> Self {
        let bank = [IIR2::new(); NUM_FILTERS];
        Self { bank }
    }
    pub fn parallel_transfer(&self, n: usize) -> Vec<f32> {
        let mut H: Vec<Complex32> = vec![CZERO; n];
        let l = Complex32 {
            re: NUM_FILTERS as f32,
            im: 0.0,
        };
        // loop over all filters first
        for (_i, filt) in self.bank.iter().enumerate() {
            let h = filt.freq_response(n);
            H.iter_mut().enumerate().for_each(|(i, x)| *x += h[i] / l);
        }

        // take norm after summing filters
        let mut out: Vec<f32> = H.iter().map(|x| x.norm()).collect();
        if out[0].is_nan() {
            out[0] = 0.0;
        }
        out
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, TS)]
#[ts(export)]
pub struct UIFilters {
    pub bank: [BPF; NUM_FILTERS],
}
impl Default for UIFilters {
    fn default() -> Self {
        let bank = [BPF::new(); NUM_FILTERS];
        Self { bank }
    }
}
impl UIFilters {
    pub fn new() -> Self {
        let bank = [BPF::new(); NUM_FILTERS];
        Self { bank }
    }
}

/// filters sent from ui to audio thread
#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub struct FiltersMessage {
    pub filters: [Option<BPF>; NUM_FILTERS],
}
impl Default for FiltersMessage {
    fn default() -> Self {
        Self {
            filters: [None; NUM_FILTERS],
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Copy, TS)]
#[ts(export)]
pub enum StereoChoice {
    Left = 0,
    Right = 1,
    Both = 2,
}

impl FromSql for StereoChoice {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        use StereoChoice::*;
        match value.as_str().unwrap() {
            "Left" => Ok(Left),
            "Right" => Ok(Right),
            "Both" => Ok(Both),
            _ => Err(rusqlite::types::FromSqlError::InvalidType),
        }
    }
}

impl IntoIterator for StereoChoice {
    type Item = StereoChoice;
    type IntoIter = std::array::IntoIter<StereoChoice, 3>;
    fn into_iter(self) -> Self::IntoIter {
        use StereoChoice::*;
        std::array::IntoIter::into_iter([Left, Right, Both].into_iter())
    }
}

impl StereoChoice {
    pub fn as_str(&self) -> &str {
        use StereoChoice::*;
        match self {
            Left => "Left",
            Right => "Right",
            Both => "Both",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AudioParams {
    pub ui_params: UIParams,
    // other stuff not from ui
    pub time: usize,
    pub dft_size: usize,
    // don't serialize all
    #[serde(skip)]
    pub filters: Filters,
    #[serde(skip)]
    pub output_spectrum: Vec<f32>,
    #[serde(skip)]
    pub noise_spectrum: Vec<f32>,
    #[serde(skip)]
    pub sdft: SDFT,
}

impl AudioParams {
    pub fn new() -> Self {
        let n = 256;
        let ui_params = UIParams::new();
        let filters = Filters::new();
        Self {
            ui_params,
            dft_size: n,
            time: 0,
            output_spectrum: vec![],
            noise_spectrum: filters.parallel_transfer(n),
            sdft: SDFT::new(n),
            filters,
        }
    }
}

impl Default for AudioParams {
    fn default() -> Self {
        let n = 256;
        let ui_params = UIParams::new();
        let filters = Filters::new();
        Self {
            ui_params,
            dft_size: n,
            time: 0,
            output_spectrum: vec![],
            noise_spectrum: filters.parallel_transfer(n),
            sdft: SDFT::new(n),
            filters,
        }
    }
}

/// stereo params includes AudioParams for each channel as well as other params that are independent of the channels
#[derive(Debug, TS, Serialize, Deserialize)]
#[ts(export)]
pub struct StereoParams {
    pub left: AudioParams,
    pub right: AudioParams,
    pub stereo_choice: StereoChoice,
    pub clean: bool,
    pub num_file_samples: usize,
    pub file_path: String,
    pub is_stereo: bool,
    pub time: usize,
}

impl StereoParams {
    pub fn new() -> Self {
        Self {
            left: AudioParams::new(),
            right: AudioParams::new(),
            stereo_choice: StereoChoice::Both,
            clean: false,
            num_file_samples: 0,
            file_path: "".to_string(),
            is_stereo: false,
            time: 0,
        }
    }
}

impl Default for StereoParams {
    fn default() -> Self {
        Self {
            left: AudioParams::new(),
            right: AudioParams::new(),
            stereo_choice: StereoChoice::Both,
            clean: false,
            num_file_samples: 0,
            file_path: "".to_string(),
            is_stereo: false,
            time: 0,
        }
    }
}

/// ui params -- states of everything in the ui, does not include everything that can be sent in a Message (file name and a few others), just the stuff that gets stored in db
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UIParams {
    pub clean: bool,
    pub left_mute: bool,
    pub right_mute: bool,
    pub stereo_choice: StereoChoice,
    pub output_gain: f32,
    pub noise_gain: f32,
    pub pre_smooth_gain: f32,
    pub post_smooth_gain: f32,
    pub filters: UIFilters,
}

impl UIParams {
    pub fn new() -> Self {
        Self {
            clean: false,
            left_mute: false,
            right_mute: false,
            stereo_choice: StereoChoice::Both,
            output_gain: 1.0,
            noise_gain: 0.0,
            pre_smooth_gain: 0.5,
            post_smooth_gain: 0.5,
            filters: UIFilters::new(),
        }
    }
}

impl Default for UIParams {
    fn default() -> Self {
        Self {
            clean: false,
            left_mute: false,
            right_mute: false,
            stereo_choice: StereoChoice::Both,
            output_gain: 1.0,
            noise_gain: 0.0,
            pre_smooth_gain: 0.5,
            post_smooth_gain: 0.5,
            filters: UIFilters::default(),
        }
    }
}
