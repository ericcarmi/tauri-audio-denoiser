use std::sync::Mutex;

use crate::{
    audio::setup_stream,
    constants::czerov,
    types::{
        AudioParams, MSender, MStream, MStreamSend, MUIReceiver, StereoAudioParams, StereoControl,
        IIR2,
    },
};
use cpal::traits::StreamTrait;
use dasp_ring_buffer::Fixed;
use rustfft::num_complex::Complex;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

#[tauri::command]
pub fn update_filters(
    bp1: Option<IIR2>,
    bp2: Option<IIR2>,
    bp3: Option<IIR2>,
    bp4: Option<IIR2>,
    bp5: Option<IIR2>,
    stereo: Option<StereoControl>,
    streamsend: State<MStreamSend>,
) {
    stereo_message(
        stereo,
        streamsend,
        ChannelMessage {
            bp1,
            bp2,
            bp3,
            bp4,
            bp5,
            ..Default::default()
        },
    );
}

#[tauri::command]
pub fn update_time(t: f32, streamsend: State<MStreamSend>, stereo: Option<StereoControl>) {
    let _ = streamsend
        .0
        .lock()
        .unwrap()
        .msender
        .0
        .lock()
        .unwrap()
        .try_send(Message {
            time: Some(t),
            ..Default::default()
        });
}

#[tauri::command]
pub fn update_clean(clean: bool, streamsend: State<MStreamSend>, stereo: Option<StereoControl>) {
    stereo_message(
        stereo,
        streamsend,
        ChannelMessage {
            clean: Some(clean),
            ..Default::default()
        },
    );
}

#[tauri::command]
pub fn update_output_gain(
    gain: f32,
    streamsend: State<MStreamSend>,
    stereo: Option<StereoControl>,
) {
    let g = (10.0_f32).powf(gain / 20.0);
    stereo_message(
        stereo,
        streamsend,
        ChannelMessage {
            output_gain: Some(g),
            ..Default::default()
        },
    );
}

#[tauri::command]
pub fn update_noise_gain(gain: f32, streamsend: State<MStreamSend>, stereo: Option<StereoControl>) {
    let g = (10.0_f32).powf(gain / 20.0) / 10.0;

    stereo_message(
        stereo,
        streamsend,
        ChannelMessage {
            noise_gain: Some(g),
            ..Default::default()
        },
    );
}

#[tauri::command]
pub fn update_pre_smooth_gain(
    gain: f32,
    streamsend: State<MStreamSend>,
    stereo: Option<StereoControl>,
) {
    stereo_message(
        stereo,
        streamsend,
        ChannelMessage {
            pre_smooth_gain: Some(gain),
            ..Default::default()
        },
    );
}

#[tauri::command]
pub fn update_post_smooth_gain(
    gain: f32,
    streamsend: State<MStreamSend>,
    stereo: Option<StereoControl>,
) {
    stereo_message(
        stereo,
        streamsend,
        ChannelMessage {
            post_smooth_gain: Some(gain),
            ..Default::default()
        },
    );
}

#[tauri::command]
pub fn update_file_path(
    path: String,
    streamsend: State<MStreamSend>,
    app_handle: AppHandle,
    stereo: Option<StereoControl>,
) {
    let (ui_tx, rx) = tauri::async_runtime::channel::<AudioUIMessage>(2);
    let (stream, tx) = setup_stream(ui_tx, app_handle, Some(path)).unwrap();
    let _ = stream.pause();
    let mtx = Mutex::new(tx);

    let mut ss = streamsend.0.lock().unwrap();
    ss.stream = MStream(Mutex::new(stream));
    ss.msender = MSender(mtx);
    ss.mreceiver = MUIReceiver(Mutex::new(rx));
}

/// the channel message is applied to left, right, or both
fn stereo_message(
    stereo: Option<StereoControl>,
    streamsend: State<MStreamSend>,
    channel_message: ChannelMessage,
) {
    if let Some(s) = stereo {
        use StereoControl::*;
        match s {
            Left => {
                let _ = streamsend
                    .0
                    .lock()
                    .unwrap()
                    .msender
                    .0
                    .lock()
                    .unwrap()
                    .try_send(Message {
                        left_channel: Some(channel_message),
                        ..Default::default()
                    });
            }
            Right => {
                let _ = streamsend
                    .0
                    .lock()
                    .unwrap()
                    .msender
                    .0
                    .lock()
                    .unwrap()
                    .try_send(Message {
                        right_channel: Some(channel_message),
                        ..Default::default()
                    });
            }
            Both => {
                let _ = streamsend
                    .0
                    .lock()
                    .unwrap()
                    .msender
                    .0
                    .lock()
                    .unwrap()
                    .try_send(Message {
                        left_channel: Some(channel_message.clone()),
                        right_channel: Some(channel_message),
                        ..Default::default()
                    });
            }
        };
    } else {
        let _ = streamsend
            .0
            .lock()
            .unwrap()
            .msender
            .0
            .lock()
            .unwrap()
            .try_send(Message {
                left_channel: Some(channel_message),
                ..Default::default()
            });
    }
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
    pub time: Option<f32>,
    pub stereo_control: StereoControl,
}

// use all None for default message to shorten other functions that send one thing at a time
impl Default for Message {
    fn default() -> Self {
        Self {
            time: None,
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
            self.recv_channel(&mut params.left, ch);
        }
        if let Some(ch) = self.right_channel {
            self.recv_channel(&mut params.right, ch);
        }

        if let Some(t) = self.time {
            params.time = (params.num_file_samples as f32 * t) as usize;
            if params.is_stereo {
                params.left.sdft.freq_history = czerov(params.left.dft_size);
                params.left.sdft.time_history =
                    Fixed::from(vec![Complex::new(0.0, 0.0); params.left.dft_size]);
                params.right.sdft.freq_history = czerov(params.right.dft_size);
                params.right.sdft.time_history =
                    Fixed::from(vec![Complex::new(0.0, 0.0); params.right.dft_size]);
            } else {
                params.left.sdft.freq_history = czerov(params.left.dft_size);
                params.left.sdft.time_history =
                    Fixed::from(vec![Complex::new(0.0, 0.0); params.left.dft_size]);
            }
        }
    }

    pub fn recv_channel(&self, channel_params: &mut AudioParams, channel_message: ChannelMessage) {
        if let Some(bp) = channel_message.bp1 {
            channel_params.filter_bank.bp1.update_coeffs(bp);
            channel_params.noise_spectrum = channel_params
                .filter_bank
                .parallel_transfer(channel_params.dft_size);
        }
        if let Some(bp) = channel_message.bp2 {
            channel_params.filter_bank.bp2.update_coeffs(bp);
            channel_params.noise_spectrum = channel_params
                .filter_bank
                .parallel_transfer(channel_params.dft_size);
        }
        if let Some(bp) = channel_message.bp3 {
            channel_params.filter_bank.bp3.update_coeffs(bp);
            channel_params.noise_spectrum = channel_params
                .filter_bank
                .parallel_transfer(channel_params.dft_size);
        }
        if let Some(bp) = channel_message.bp4 {
            channel_params.filter_bank.bp5.update_coeffs(bp);
            channel_params.noise_spectrum = channel_params
                .filter_bank
                .parallel_transfer(channel_params.dft_size);
        }
        if let Some(bp) = channel_message.bp5 {
            channel_params.filter_bank.bp5.update_coeffs(bp);
            channel_params.noise_spectrum = channel_params
                .filter_bank
                .parallel_transfer(channel_params.dft_size);
        }
        if let Some(c) = channel_message.clean {
            channel_params.clean = c;
            channel_params.sdft.freq_history = czerov(channel_params.dft_size);
            channel_params.sdft.time_history =
                Fixed::from(vec![Complex::new(0.0, 0.0); channel_params.dft_size]);
        }
        if let Some(g) = channel_message.output_gain {
            channel_params.output_gain = g;
        }
        if let Some(g) = channel_message.noise_gain {
            channel_params.noise_gain = g;
        }
        if let Some(g) = channel_message.pre_smooth_gain {
            channel_params.pre_smooth_gain = g;
        }
        if let Some(g) = channel_message.post_smooth_gain {
            channel_params.post_smooth_gain = g;
        }
    }
}

#[tauri::command]
pub fn get_is_stereo(streamsend: State<MStreamSend>) -> Result<AudioUIMessage, String> {
    let r = streamsend
        .0
        .lock()
        .unwrap()
        .mreceiver
        .0
        .lock()
        .unwrap()
        .try_recv();
    if r.is_ok() {
        Ok(r.unwrap())
    } else {
        r.map_err(|e| e.to_string())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AudioUIMessage {
    pub spectrum: Option<Vec<f32>>,
    pub is_stereo: Option<bool>,
}

impl Default for AudioUIMessage {
    fn default() -> Self {
        Self {
            spectrum: None,
            is_stereo: None,
        }
    }
}
