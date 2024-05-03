use std::sync::Mutex;

use crate::{
    audio::setup_stream,
    constants::czerov,
    types::{
        AudioParams, FilterBank, MSender, MStream, MStreamSend, MUIReceiver, StereoChoice,
        StereoParams, BPF, IIR2,
    },
};
use cpal::traits::StreamTrait;
use dasp_ring_buffer::Fixed;
use rustfft::num_complex::Complex;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

/// this is to load params directly from server, the ones that are setup in the stream, the audio params...want to keep them synced with UI on startup, so this will just be called onMount from frontend
#[tauri::command]
pub async fn init_audio_params_from_server(
    streamsend: State<'_, MStreamSend>,
) -> Result<(), String> {
    // let left_mute = get_mute(StereoChoice::Left).await.unwrap();
    // let right_mute = get_mute(StereoChoice::Right).await.unwrap();

    // stereo_message(
    //     Some(StereoChoice::Left),
    //     streamsend.clone(),
    //     Some(ChannelMessage {
    //         mute: Some(left_mute),
    //         ..Default::default()
    //     }),
    // );
    // stereo_message(
    //     Some(StereoChoice::Right),
    //     streamsend,
    //     Some(ChannelMessage {
    //         mute: Some(right_mute),
    //         ..Default::default()
    //     }),
    // );

    Ok(())
}

#[tauri::command]
pub fn message_filters(
    stereo_choice: StereoChoice,
    filter_bank_message: FilterBankMessage,
    streamsend: State<MStreamSend>,
) {
    stereo_message(
        stereo_choice,
        streamsend,
        Some(ChannelMessage {
            filter_bank: Some(filter_bank_message),
            ..Default::default()
        }),
    );
}

#[tauri::command]
pub fn message_time(t: f32, streamsend: State<MStreamSend>) {
    let _ = streamsend
        .0
        .lock()
        .unwrap()
        .msender
        .0
        .lock()
        .unwrap()
        .try_send(UIAudioMessage {
            time: Some(t),
            ..Default::default()
        });
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
pub fn message_mute(mute: bool, streamsend: State<MStreamSend>, stereo_choice: StereoChoice) {
    stereo_message(
        stereo_choice,
        streamsend,
        Some(ChannelMessage {
            mute: Some(mute),
            ..Default::default()
        }),
    );
}

#[tauri::command]
pub fn message_output_gain(gain: f32, streamsend: State<MStreamSend>, stereo_choice: StereoChoice) {
    let g = (10.0_f32).powf(gain / 20.0);
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
    let g = (10.0_f32).powf(gain / 20.0) / 10.0;

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
pub fn message_file_path(path: String, streamsend: State<MStreamSend>, app_handle: AppHandle) {
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

// would prefer to have this derived from FilterBank, but would have to do with macros to get Options for each item splayed out
#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub struct FilterBankMessage {
    pub bp1: Option<BPF>,
    pub bp2: Option<BPF>,
    pub bp3: Option<BPF>,
    pub bp4: Option<BPF>,
    pub bp5: Option<BPF>,
}

/// a message from a single channel (left or right)
#[derive(Clone, Debug, Copy)]
pub struct ChannelMessage {
    pub time: Option<f32>,
    pub clean: Option<bool>,
    pub mute: Option<bool>,
    pub output_gain: Option<f32>,
    pub noise_gain: Option<f32>,
    pub pre_smooth_gain: Option<f32>,
    pub post_smooth_gain: Option<f32>,
    pub filter_bank: Option<FilterBankMessage>,
}

impl Default for ChannelMessage {
    fn default() -> Self {
        // let filter_bank = FilterBank{bp}
        Self {
            time: None,
            clean: None,
            mute: None,
            output_gain: None,
            noise_gain: None,
            pre_smooth_gain: None,
            post_smooth_gain: None,
            filter_bank: None,
        }
    }
}

// use options with everything...a little annoying but then use None when passing to ignore most sub-structs
// this struct is for messages sent from UI to audio thread
#[derive(Clone, Debug)]
pub struct UIAudioMessage {
    pub left_channel: Option<ChannelMessage>,
    pub right_channel: Option<ChannelMessage>,
    pub file_path: Option<String>,
    pub time: Option<f32>,
    pub stereo_choice: Option<StereoChoice>,
    pub clean: Option<bool>,
    pub export: Option<bool>,
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
        if let Some(fb) = channel_message.filter_bank {
            if let Some(bp) = fb.bp1 {
                channel_params.filter_bank.bp1.update_coeffs(bp.into());
                channel_params.noise_spectrum = channel_params
                    .filter_bank
                    .parallel_transfer(channel_params.dft_size);
            }
            if let Some(bp) = fb.bp2 {
                channel_params.filter_bank.bp2.update_coeffs(bp.into());
                channel_params.noise_spectrum = channel_params
                    .filter_bank
                    .parallel_transfer(channel_params.dft_size);
            }
            if let Some(bp) = fb.bp3 {
                channel_params.filter_bank.bp3.update_coeffs(bp.into());
                channel_params.noise_spectrum = channel_params
                    .filter_bank
                    .parallel_transfer(channel_params.dft_size);
            }
            if let Some(bp) = fb.bp4 {
                channel_params.filter_bank.bp5.update_coeffs(bp.into());
                channel_params.noise_spectrum = channel_params
                    .filter_bank
                    .parallel_transfer(channel_params.dft_size);
            }
            if let Some(bp) = fb.bp5 {
                channel_params.filter_bank.bp5.update_coeffs(bp.into());
                channel_params.noise_spectrum = channel_params
                    .filter_bank
                    .parallel_transfer(channel_params.dft_size);
            }
        }
        // if let Some(m) = channel_message.mute {
        //     channel_params.mute = m;
        // }
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

/// message sent from audio thread to ui
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AudioUIMessage {
    pub spectrum: Option<Vec<f32>>,
    pub is_stereo: Option<bool>,
    pub is_processing: Option<bool>,
    pub processing_percentage: Option<f32>,
}

impl Default for AudioUIMessage {
    fn default() -> Self {
        Self {
            spectrum: None,
            is_stereo: None,
            is_processing: None,
            processing_percentage: None,
        }
    }
}
