// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::sync::Mutex;

use cpal::traits::StreamTrait;
use tauri::{Manager, State};
mod audio;
use audio::*;
mod types;
use types::*;
mod constants;
mod fourier;
use fourier::*;
mod server;
use server::*;

fn main() {
    tauri::Builder::default()
        .manage(MStreamSend({
            let (stream, tx) = setup_stream().unwrap();
            let _ = stream.pause();
            let mtx = Mutex::new(tx);

            Mutex::new(StreamSend {
                stream: MStream(Mutex::new(stream)),
                msender: MSender(mtx),
            })
        }))
        .manage(MFilterBank({
            let fbank = FilterBank::new();
            Mutex::new(fbank)
        }))
        .invoke_handler(tauri::generate_handler![
            play_stream,
            pause_stream,
            update_filters,
            update_time,
            get_stft_data,
            get_time_onefft,
            get_integer,
            get_file_fft,
            set_file_fft,
        ])
        .setup(|app| {
            let mainwindow = app.get_window("main").unwrap();
            let _ = mainwindow.set_always_on_top(true);
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
