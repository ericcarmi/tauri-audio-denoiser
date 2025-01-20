#![allow(non_snake_case)]
use std::{path::PathBuf, str::FromStr, sync::Mutex};

use crate::{
    audio::{calculate_fingerprint, setup_stream},
    constants::{czerov, from_log, NUM_FILTERS},
    types::{
        AudioParams, MSender, MStream, MStreamSend, MUIReceiver, StereoChoice, StereoParams, BPF,
        IIR2,
    },
};
use cpal::traits::StreamTrait;
use dasp_ring_buffer::Fixed;
use rustfft::num_complex::Complex;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State, Window};

// any reason not to use a single function for most of these? would be a lot less code, same effect

/// this is to load params directly from server, the ones that are setup in the stream, the audio params...want to keep them synced with UI on startup, so this will just be called onMount from frontend
#[tauri::command]
pub async fn message_all(
    streamsend: State<'_, MStreamSend>,
    stereo_choice: StereoChoice,
    clean: bool,
    left_mute: bool,
    right_mute: bool,
    output_gain: f32,
    noise_gain: f32,
    pre_smooth_gain: f32,
    post_smooth_gain: f32,
    filters: [Option<BPF>; NUM_FILTERS],
) -> Result<(), String> {
    stereo_message(
        stereo_choice,
        streamsend,
        Some(ChannelMessage {
            clean: Some(clean),
            left_mute: Some(left_mute),
            right_mute: Some(right_mute),
            output_gain: Some(from_log(output_gain)),
            noise_gain: Some(from_log(noise_gain)),
            pre_smooth_gain: Some(pre_smooth_gain),
            post_smooth_gain: Some(post_smooth_gain),
            filters: Some(filters),
            ..Default::default()
        }),
    );

    Ok(())
}

#[tauri::command]
pub fn message_filters(
    stereo_choice: StereoChoice,
    index: usize,
    gain: f32,
    freq: f32,
    Q: f32,
    streamsend: State<MStreamSend>,
) {
    if index >= NUM_FILTERS {
        return;
    }
    let gain = from_log(gain);

    let mut filters = [None; NUM_FILTERS];
    let bpf = BPF {
        gain: from_log(gain),
        freq,
        Q,
    };
    filters[index] = Some(bpf);

    stereo_message(
        stereo_choice,
        streamsend,
        Some(ChannelMessage {
            filters: Some(filters),
            ..Default::default()
        }),
    );
}

/// this only works after stream has started, because the time variable created when stream is setup...something like that? maybe it isn't, i forgot
#[tauri::command]
pub fn message_time(time: f32, streamsend: State<MStreamSend>) {
    let _ = streamsend
        .0
        .lock()
        .unwrap()
        .msender
        .0
        .lock()
        .unwrap()
        .try_send(UIAudioMessage {
            time: Some(time),
            is_looping: Some(false),
            ..Default::default()
        });
}

#[tauri::command]
pub fn message_loop_time(loop_time: usize, loop_length: usize, streamsend: State<MStreamSend>) {
    let _ = streamsend
        .0
        .lock()
        .unwrap()
        .msender
        .0
        .lock()
        .unwrap()
        .try_send(UIAudioMessage {
            loop_start_time: Some(loop_time),
            loop_length: Some(loop_length),
            is_looping: Some(true),
            ..Default::default()
        });
}
#[tauri::command]
pub fn message_fingerprint(
    streamsend: State<MStreamSend>,
    start: usize,
    len: usize,
    app_handle: AppHandle,
    file_name: &str,
) {
    let file = app_handle
        .path_resolver()
        .resource_dir()
        .expect("failed to open resource dir")
        .join("assets")
        .join(file_name);
    // let _ = streamsend
    //     .0
    //     .lock()
    //     .unwrap()
    //     .msender
    //     .0
    //     .lock()
    //     .unwrap()
    //     .try_send(UIAudioMessage {
    //         fingerprint: Some(true),
    //         start_fingerprint: Some(start),
    //         length_fingerprint: Some(len),
    //         file_path: Some(file),
    //         ..Default::default()
    //     });
    // shouldn't be a stream message, don't need the audio stream to be running
    let _ = streamsend
        .0
        .lock()
        .unwrap()
        .stream
        .0
        .lock()
        .unwrap()
        .pause();
    let w = app_handle.get_window("main").unwrap();
    calculate_fingerprint(file, start, len, w);
}

#[tauri::command]
pub fn message_clean(clean: bool, streamsend: State<MStreamSend>, stereo_choice: StereoChoice) {
    stereo_message(
        stereo_choice,
        streamsend,
        Some(ChannelMessage {
            clean: Some(clean),
            ..Default::default()
        }),
    );
}
#[tauri::command]
pub fn message_left_mute(mute: bool, streamsend: State<MStreamSend>, stereo_choice: StereoChoice) {
    stereo_message(
        stereo_choice,
        streamsend,
        Some(ChannelMessage {
            left_mute: Some(mute),
            ..Default::default()
        }),
    );
}

#[tauri::command]
pub fn message_right_mute(mute: bool, streamsend: State<MStreamSend>, stereo_choice: StereoChoice) {
    stereo_message(
        stereo_choice,
        streamsend,
        Some(ChannelMessage {
            right_mute: Some(mute),
            ..Default::default()
        }),
    );
}

#[tauri::command]
pub fn message_output_gain(gain: f32, streamsend: State<MStreamSend>, stereo_choice: StereoChoice) {
    let g = from_log(gain);
    stereo_message(
        stereo_choice,
        streamsend,
        Some(ChannelMessage {
            output_gain: Some(g),
            ..Default::default()
        }),
    );
}

#[tauri::command]
pub fn message_noise_gain(gain: f32, streamsend: State<MStreamSend>, stereo_choice: StereoChoice) {
    let g = from_log(gain);

    stereo_message(
        stereo_choice,
        streamsend,
        Some(ChannelMessage {
            noise_gain: Some(g),
            ..Default::default()
        }),
    );
}

#[tauri::command]
pub fn message_pre_smooth_gain(
    gain: f32,
    streamsend: State<MStreamSend>,
    stereo_choice: StereoChoice,
) {
    stereo_message(
        stereo_choice,
        streamsend,
        Some(ChannelMessage {
            pre_smooth_gain: Some(gain),
            ..Default::default()
        }),
    );
}

#[tauri::command]
pub fn message_post_smooth_gain(
    gain: f32,
    streamsend: State<MStreamSend>,
    stereo_choice: StereoChoice,
) {
    stereo_message(
        stereo_choice,
        streamsend,
        Some(ChannelMessage {
            post_smooth_gain: Some(gain),
            ..Default::default()
        }),
    );
}

#[tauri::command]
pub fn message_file_path(
    path: String,
    streamsend: State<MStreamSend>,
    app_handle: AppHandle,
    window: Window,
) {
    let _ = streamsend
        .0
        .lock()
        .unwrap()
        .stream
        .0
        .lock()
        .unwrap()
        .pause();
    let (ui_tx, rx) = tauri::async_runtime::channel::<AudioUIMessage>(2);
    let path = PathBuf::from_str(path.as_str()).expect("bad path");
    let (stream, tx) = setup_stream(ui_tx, app_handle, Some(path), window).unwrap();
    let _ = stream.pause();
    let mtx = Mutex::new(tx);

    let mut ss = streamsend.0.lock().unwrap();
    ss.stream = MStream(Mutex::new(stream));
    ss.msender = MSender(mtx);
    ss.mreceiver = MUIReceiver(Mutex::new(rx));
}

/// the channel message is applied to left, right, or both
fn stereo_message(
    stereo_choice: StereoChoice,
    streamsend: State<MStreamSend>,
    channel_message: Option<ChannelMessage>,
) {
    use StereoChoice::*;
    match stereo_choice {
        Left => {
            let _ = streamsend
                .0
                .lock()
                .unwrap()
                .msender
                .0
                .lock()
                .unwrap()
                .try_send(UIAudioMessage {
                    left_channel: channel_message,
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
                .try_send(UIAudioMessage {
                    right_channel: channel_message,
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
                .try_send(UIAudioMessage {
                    left_channel: channel_message,
                    right_channel: channel_message,
                    ..Default::default()
                });
        }
    };
}

/// a message from a single channel (left or right)
#[derive(Clone, Debug, Copy)]
pub struct ChannelMessage {
    pub time: Option<f32>,
    pub loop_length: Option<f32>,
    pub clean: Option<bool>,
    pub left_mute: Option<bool>,
    pub right_mute: Option<bool>,
    pub output_gain: Option<f32>,
    pub noise_gain: Option<f32>,
    pub pre_smooth_gain: Option<f32>,
    pub post_smooth_gain: Option<f32>,
    pub filters: Option<[Option<BPF>; NUM_FILTERS]>,
}

impl Default for ChannelMessage {
    fn default() -> Self {
        // let filter_bank = FilterBank{bp}
        Self {
            time: None,
            loop_length: None,
            clean: None,
            left_mute: None,
            right_mute: None,
            output_gain: None,
            noise_gain: None,
            pre_smooth_gain: None,
            post_smooth_gain: None,
            filters: None,
        }
    }
}

// use options with everything...a little annoying but then use None when passing to ignore most sub-structs
// this struct is for messages sent from UI to audio thread
#[derive(Clone, Debug)]
pub struct UIAudioMessage {
    pub left_channel: Option<ChannelMessage>,
    pub right_channel: Option<ChannelMessage>,
    pub file_path: Option<PathBuf>,
    pub stereo_choice: Option<StereoChoice>,
    pub clean: Option<bool>,
    pub export: Option<bool>,
    pub time: Option<f32>,
    pub loop_start_time: Option<usize>,
    pub loop_length: Option<usize>,
    pub is_looping: Option<bool>,
    // don't really need to send bool, it is true when message is received?
    pub fingerprint: Option<bool>,
    pub start_fingerprint: Option<usize>,
    pub length_fingerprint: Option<usize>,
}

// use all None for default message to shorten other functions that send one thing at a time
impl Default for UIAudioMessage {
    fn default() -> Self {
        Self {
            time: None,
            file_path: None,
            clean: None,
            stereo_choice: None,
            left_channel: None,
            right_channel: None,
            export: None,
            loop_start_time: None,
            loop_length: None,
            is_looping: None,
            fingerprint: None,
            start_fingerprint: None,
            length_fingerprint: None,
        }
    }
}

impl UIAudioMessage {
    pub fn receive(&self, params: &mut StereoParams) {
        // apply controls to channels
        use StereoChoice::*;
        match params.stereo_choice {
            Left => {
                if let Some(ch) = self.left_channel {
                    self.recv_channel(&mut params.left, ch);
                    if let Some(c) = ch.clean {
                        params.clean = c;
                        params.left.sdft.freq_history = czerov(params.left.dft_size);
                        params.left.sdft.time_history =
                            Fixed::from(vec![Complex::new(0.0, 0.0); params.left.dft_size]);
                    }
                }
            }
            Right => {
                if let Some(ch) = self.right_channel {
                    self.recv_channel(&mut params.right, ch);
                    if let Some(c) = ch.clean {
                        params.clean = c;
                        params.right.sdft.freq_history = czerov(params.right.dft_size);
                        params.right.sdft.time_history =
                            Fixed::from(vec![Complex::new(0.0, 0.0); params.right.dft_size]);
                    }
                }
            }
            Both => {
                if let Some(ch) = self.left_channel {
                    self.recv_channel(&mut params.left, ch);
                    if let Some(c) = ch.clean {
                        params.clean = c;
                        params.left.sdft.freq_history = czerov(params.left.dft_size);
                        params.left.sdft.time_history =
                            Fixed::from(vec![Complex::new(0.0, 0.0); params.left.dft_size]);
                    }
                }
                if let Some(ch) = self.right_channel {
                    self.recv_channel(&mut params.right, ch);
                    if let Some(c) = ch.clean {
                        params.clean = c;
                        params.right.sdft.freq_history = czerov(params.right.dft_size);
                        params.right.sdft.time_history =
                            Fixed::from(vec![Complex::new(0.0, 0.0); params.right.dft_size]);
                    }
                }
            }
        }

        // this is where time is handled differently? was there a reason? forgot...
        if let Some(t) = self.time {
            params.time = (t) as usize;
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
        if let Some(t) = self.loop_start_time {
            params.loop_start_time = t;
            params.time = t;
        }
        if let Some(t) = self.loop_length {
            params.loop_length = t;
        }
        if let Some(t) = self.is_looping {
            params.is_looping = t;
        }
        if self.fingerprint.is_some()
            && self.file_path.is_some()
            && self.start_fingerprint.is_some()
            && self.length_fingerprint.is_some()
        {
            // audio::calculate_fingerprint(
            //     self.file_path.clone().unwrap(),
            //     self.start_fingerprint.unwrap(),
            //     self.length_fingerprint.unwrap(),
            // );
        }
    }

    pub fn recv_channel(&self, channel_params: &mut AudioParams, channel_message: ChannelMessage) {
        if let Some(msg) = channel_message.filters {
            for (i, filter) in msg.iter().enumerate() {
                if let Some(f) = filter {
                    let iir: IIR2 = Into::<IIR2>::into(*f);
                    channel_params.filters.bank[i].update_coeffs(iir);
                    channel_params.noise_spectrum = channel_params
                        .filters
                        .parallel_transfer(channel_params.dft_size);
                }
            }
        }
        if let Some(m) = channel_message.left_mute {
            channel_params.ui_params.left_mute = m;
        }
        if let Some(m) = channel_message.right_mute {
            channel_params.ui_params.right_mute = m;
        }
        if let Some(g) = channel_message.output_gain {
            channel_params.ui_params.output_gain = g;
        }
        if let Some(g) = channel_message.noise_gain {
            channel_params.ui_params.noise_gain = g;
        }
        if let Some(g) = channel_message.pre_smooth_gain {
            channel_params.ui_params.pre_smooth_gain = g;
        }
        if let Some(g) = channel_message.post_smooth_gain {
            channel_params.ui_params.post_smooth_gain = g;
        }
    }
}

/// message sent from audio thread to ui
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AudioUIMessage {
    pub spectrum: Option<Vec<f32>>,
    pub is_stereo: Option<bool>,
    pub is_processing: Option<bool>,
    pub processing_percentage: Option<f32>,
    pub time: Option<f32>,
}

impl AudioUIMessage {
    pub fn name() -> &'static str {
        "audioui_message"
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct FingerprintMessage {
    pub spectrum: Option<Vec<f32>>,
    pub filters: [Option<BPF>; NUM_FILTERS],
}

impl FingerprintMessage {
    pub fn name() -> &'static str {
        "fingerprint_message"
    }
}
