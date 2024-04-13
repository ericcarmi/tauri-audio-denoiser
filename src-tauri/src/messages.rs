use std::sync::Mutex;

use crate::{
    audio::setup_stream,
    types::{
        ChannelMessage, MSender, MStream, MStreamSend, MUIReceiver, Message, StereoControl, IIR2,
    },
};
use cpal::traits::StreamTrait;
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
    stereo_message(
        stereo,
        streamsend,
        ChannelMessage {
            time: Some(t),
            ..Default::default()
        },
    );
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
    let (ui_tx, rx) = tauri::async_runtime::channel::<Vec<f32>>(2);
    let (stream, tx) = setup_stream(ui_tx, app_handle, Some(path)).unwrap();
    let _ = stream.pause();
    let mtx = Mutex::new(tx);

    let mut ss = streamsend.0.lock().unwrap();
    ss.stream = MStream(Mutex::new(stream));
    ss.msender = MSender(mtx);
    ss.mreceiver = MUIReceiver(Mutex::new(rx));
}

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
