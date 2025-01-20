// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]
// #![allow(dead_code)]
use cpal::traits::StreamTrait;
use denoiser::{audio::device_sample_rate, sql::create_db};
use std::{fs::File, sync::Mutex};
use tauri::{AppHandle, Manager, State};
mod audio;
use audio::*;
mod types;
use types::*;
mod constants;
mod fourier;
use fourier::*;
mod errors;
mod messages;
mod sdft;
use messages::*;
mod file_io;
mod settings;
use file_io::*;
// use log::info;
mod sql;
use simplelog::*;
use sql::*;

fn main() {
    let app = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            windows_explorer,
            play_stream,
            pause_stream,
            get_stft_data,
            get_time_onefft,
            get_time_data,
            constants::get_num_filters,
            message_filters,
            message_time,
            message_clean,
            message_output_gain,
            message_noise_gain,
            message_pre_smooth_gain,
            message_post_smooth_gain,
            message_file_path,
            message_left_mute,
            message_right_mute,
            message_loop_time,
            message_fingerprint,
            message_all,
            process_export,
            sql_theme,
            sql_theme_name,
            sql_settings,
            sql_ui_params,
            sql_update_ui_params,
            sql_filter_bank,
            sql_update_filter_bank,
            sql_update_noise_gain,
            sql_update_output_gain,
            sql_update_pre_smooth_gain,
            sql_update_post_smooth_gain,
            sql_update_clean,
            sql_update_left_mute,
            sql_update_right_mute,
        ])
        .setup(|app| {
            let mainwindow = app.get_window("main").unwrap();
            // let _ = mainwindow.set_always_on_top(true);
            let app_handle = app.app_handle();

            let db_path = app_handle
                .path_resolver()
                .app_local_data_dir()
                .expect("AppData\\Local\\denoiser should exist")
                .join("db.sqlite");

            let exists = std::fs::exists(&db_path);
            if exists.is_err() || !exists.unwrap() {
                println!("CREATE DATABASE");
                let _r = create_db(db_path);
            }

            // let c = ComponentColors::as_slice();
            // println!("{:?}", c);

            let p = app_handle
                .path_resolver()
                .app_local_data_dir()
                .expect("AppData\\Local\\denoiser should exist")
                .join("log.txt");

            let _w = WriteLogger::init(
                LevelFilter::Info,
                Config::default(),
                File::create(p).unwrap(),
            );

            let m = mainwindow.available_monitors();
            let _ = mainwindow.set_position(*m.unwrap()[0].position());
            let window = app_handle.get_window("main").unwrap();

            let mss = MStreamSend({
                let (tx_ui, rx_ui) = tauri::async_runtime::channel::<AudioUIMessage>(2);
                let (stream, tx) =
                    setup_stream(tx_ui.clone(), app_handle, None, window.clone()).unwrap();
                let _ = stream.pause();
                let mtx = Mutex::new(tx);

                Mutex::new(StreamSend {
                    stream: MStream(Mutex::new(stream)),
                    msender: MSender(mtx),
                    mreceiver: MUIReceiver(Mutex::new(rx_ui)),
                    mtx_ui: MAudioSender(Mutex::new(tx_ui)),
                })
            });

            let sr = device_sample_rate().unwrap().0;
            let _ = window.clone().emit("update_sampling_rate", sr);

            let _ = app.manage(mss);

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application");
    app.run(|_app_handle, event| match event {
        tauri::RunEvent::ExitRequested { .. } => {
            // can get api from brackets ^
        }
        _ => {}
    });
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
fn windows_explorer(app_handle: AppHandle) {
    let p = app_handle
        .path_resolver()
        .app_local_data_dir()
        .expect("applocal should exist");
    std::process::Command::new("explorer")
        .arg(p)
        .output()
        .expect("failed to launch explorer window for AppData\\Local\\denoiser");
}
