use std::sync::Mutex;

use cpal::Stream;

pub struct MStream(pub Mutex<Stream>);

pub struct MFilterBank(pub Mutex<FilterBank>);

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

#[derive(Clone)]
pub struct FilterBank {
    pub coeffs: Vec<f32>,
    // pub input_history: Bounded<Vec<f32>>,
}

impl FilterBank {
    pub fn new() -> Self {
        // let rb = dasp_ring_buffer::Bounded::from(vec![0.0; 3]);

        Self {
            coeffs: vec![0.5, 0.5],
            // input_history: rb,
        }
    }
}

// use options with everything...a little annoying but then use None when passing to ignore most sub-structs
#[derive(Clone)]
pub struct Message {
    pub filter_bank: Option<FilterBank>,
    pub time: Option<f32>,
    // pub wav_file: Option<WavReader<BufReader<File>>>,
}
