// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::sync::Mutex;

use cpal::traits::StreamTrait;
use hound::{WavReader, WavWriter};
use tauri::State;
mod oscillator;
use oscillator::*;

fn main() {
    tauri::Builder::default()
        .manage(AppState({
            let (stream, tx) = oscillator::stream_setup_for().unwrap();
            let _ = stream.pause();
            let mtx = Mutex::new(tx);
            let fbank = FilterBank::new();

            Mutex::new(AppStruct {
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
fn play_wav(path: &str, appstate: State<AppState>) {
    appstate.0.lock().unwrap().stream.0.lock().unwrap().play();
}

#[tauri::command]
fn stop(appstate: State<AppState>) {
    appstate.0.lock().unwrap().stream.0.lock().unwrap().pause();
    // let stream = mstream.0.lock().unwrap();
    // let _ = stream.pause();
}
