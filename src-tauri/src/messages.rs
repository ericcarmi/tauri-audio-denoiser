use std::sync::Mutex;

use crate::{
    audio::setup_stream,
    types::{MSender, MStream, MStreamSend, MUIReceiver, Message, StreamSend, IIR2},
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
    streamsend: State<MStreamSend>,
) {
    let _ = streamsend
        .0
        .lock()
        .unwrap()
        .msender
        .0
        .lock()
        .unwrap()
        .try_send(Message {
            bp1,
            bp2,
            bp3,
            bp4,
            bp5,
            ..Default::default()
        });
}

#[tauri::command]
pub fn update_time(t: f32, streamsend: State<MStreamSend>) {
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
pub fn update_clean(clean: bool, streamsend: State<MStreamSend>) {
    let _ = streamsend
        .0
        .lock()
        .unwrap()
        .msender
        .0
        .lock()
        .unwrap()
        .try_send(Message {
            clean: Some(clean),
            ..Default::default()
        });
}

#[tauri::command]
pub fn update_bypass(bypass: bool, index: usize, streamsend: State<MStreamSend>) {
    let mut bp = vec![None; 5];
    bp[index] = Some(bypass);
    let _ = streamsend
        .0
        .lock()
        .unwrap()
        .msender
        .0
        .lock()
        .unwrap()
        .try_send(Message {
            bypass: Some(bp),
            ..Default::default()
        });
}

#[tauri::command]
pub fn update_output_gain(gain: f32, streamsend: State<MStreamSend>) {
    let g = (10.0_f32).powf(gain / 20.0);
    let _ = streamsend
        .0
        .lock()
        .unwrap()
        .msender
        .0
        .lock()
        .unwrap()
        .try_send(Message {
            output_gain: Some(g),
            ..Default::default()
        });
}

#[tauri::command]
pub fn update_noise_gain(gain: f32, streamsend: State<MStreamSend>) {
    let g = (10.0_f32).powf(gain / 20.0) / 10.0;

    let _ = streamsend
        .0
        .lock()
        .unwrap()
        .msender
        .0
        .lock()
        .unwrap()
        .try_send(Message {
            noise_gain: Some(g),
            ..Default::default()
        });
}

#[tauri::command]
pub fn update_pre_smooth_gain(gain: f32, streamsend: State<MStreamSend>) {
    let _ = streamsend
        .0
        .lock()
        .unwrap()
        .msender
        .0
        .lock()
        .unwrap()
        .try_send(Message {
            pre_smooth_gain: Some(gain),
            ..Default::default()
        });
}

#[tauri::command]
pub fn update_post_smooth_gain(gain: f32, streamsend: State<MStreamSend>) {
    let _ = streamsend
        .0
        .lock()
        .unwrap()
        .msender
        .0
        .lock()
        .unwrap()
        .try_send(Message {
            post_smooth_gain: Some(gain),
            ..Default::default()
        });
}

#[tauri::command]
pub fn update_noise_variance(gain: f32, streamsend: State<MStreamSend>) {
    let _ = streamsend
        .0
        .lock()
        .unwrap()
        .msender
        .0
        .lock()
        .unwrap()
        .try_send(Message {
            noise_variance: Some(gain),
            ..Default::default()
        });
}

#[tauri::command]
pub fn update_file_path(path: String, streamsend: State<MStreamSend>, app_handle: AppHandle) {
    let (ui_tx, rx) = tauri::async_runtime::channel::<Vec<f32>>(2);
    let (stream, tx) = setup_stream(ui_tx, app_handle, Some(path)).unwrap();
    let _ = stream.pause();
    let mtx = Mutex::new(tx);

    let mut ss = streamsend.0.lock().unwrap();
    ss.stream = MStream(Mutex::new(stream));
    ss.msender = MSender(mtx);
    ss.mreceiver = MUIReceiver(Mutex::new(rx));
}
