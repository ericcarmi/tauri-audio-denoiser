// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]
use cpal::traits::StreamTrait;
use std::sync::Mutex;
use tauri::{Manager, State};
mod audio;
use audio::*;
mod types;
use types::*;
mod constants;
use constants::*;
mod fourier;
use fourier::*;
mod sdft;
mod server;
use server::*;
mod settings;
use settings::*;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            play_stream,
            pause_stream,
            update_filters,
            update_time,
            update_clean,
            get_stft_data,
            get_time_onefft,
            get_time_data,
            get_file_fft,
            set_file_fft,
            get_fft_plot_data,
            save_global_state,
            get_global_state,
            save_bpf_gain,
            save_bpf_freq,
            save_bpf_Q,
            update_bypass,
            update_output_gain,
            update_noise_gain,
            save_output_gain,
            save_noise_gain,
            get_noise_gain,
            get_output_gain,
            update_pre_smooth_gain,
            update_post_smooth_gain,
            update_noise_variance,
            get_settings,
            save_settings,
        ])
        .setup(|app| {
            let mainwindow = app.get_window("main").unwrap();
            let _ = mainwindow.set_always_on_top(true);

            let app_handle = app.app_handle();

            let mss = MStreamSend({
                let (ui_tx, rx) = tauri::async_runtime::channel::<Vec<f32>>(2);
                let (stream, tx) = setup_stream(ui_tx, app_handle).unwrap();
                let _ = stream.pause();
                let mtx = Mutex::new(tx);

                Mutex::new(StreamSend {
                    stream: MStream(Mutex::new(stream)),
                    msender: MSender(mtx),
                    mreceiver: MUIReceiver(Mutex::new(rx)),
                })
            });

            let _ = app.manage(mss);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn play_stream(streamsend: State<MStreamSend>) {
    let _ = streamsend.0.lock().unwrap().stream.0.lock().unwrap().play();
}

#[tauri::command]
fn pause_stream(streamsend: State<MStreamSend>) {
    let _ = streamsend
        .0
        .lock()
        .unwrap()
        .stream
        .0
        .lock()
        .unwrap()
        .pause();
}

#[tauri::command]
fn get_fft_plot_data(streamsend: State<MStreamSend>) -> Result<Vec<f32>, String> {
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
        Err("recv error".into())
    }
}

#[derive(Clone, serde::Serialize)]
struct Payload {
    message: String,
}
