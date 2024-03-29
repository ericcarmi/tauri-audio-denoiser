// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::sync::Mutex;

use cpal::traits::StreamTrait;
use tauri::State;
mod oscillator;
use oscillator::*;
mod types;
use types::*;
mod constants;

fn main() {
    tauri::Builder::default()
        .manage(MStreamSend({
            let (stream, tx) = oscillator::stream_setup_for().unwrap();
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
        .invoke_handler(tauri::generate_handler![play_wav, stop, update_filters])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn play_wav(streamsend: State<MStreamSend>) {
    let _ = streamsend.0.lock().unwrap().stream.0.lock().unwrap().play();
}

#[tauri::command]
fn stop(streamsend: State<MStreamSend>) {
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
