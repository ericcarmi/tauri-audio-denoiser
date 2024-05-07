// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]
// #![allow(dead_code)]
use cpal::traits::StreamTrait;
use std::{fs::File, sync::Mutex};
use tauri::{Manager, State};
mod audio;
use audio::*;
mod types;
use types::*;
mod constants;
mod fourier;
use fourier::*;
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
            play_stream,
            pause_stream,
            get_stft_data,
            get_time_onefft,
            get_time_data,
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

            // let c = ComponentColors::as_slice();
            // println!("{:?}", c);

            let p = app_handle
                .path_resolver()
                .resource_dir()
                .expect("failed to resolve resource")
                .into_os_string()
                .into_string()
                .unwrap();

            #[cfg(target_os = "windows")]
            let _w = WriteLogger::init(
                LevelFilter::Info,
                Config::default(),
                File::create(p.to_owned() + "\\text.log").unwrap(),
            );

            #[cfg(not(target_os = "windows"))]
            let _w = WriteLogger::init(
                LevelFilter::Info,
                Config::default(),
                File::create(p.to_owned() + "/text.log").unwrap(),
            );

            let m = mainwindow.available_monitors();
            let _ = mainwindow.set_position(*m.unwrap()[0].position());
            let window = app_handle.get_window("main").unwrap();

            let mss = MStreamSend({
                let (tx_ui, rx_ui) = tauri::async_runtime::channel::<AudioUIMessage>(2);
                let (stream, tx) = setup_stream(tx_ui.clone(), app_handle, None, window).unwrap();
                let _ = stream.pause();
                let mtx = Mutex::new(tx);

                Mutex::new(StreamSend {
                    stream: MStream(Mutex::new(stream)),
                    msender: MSender(mtx),
                    mreceiver: MUIReceiver(Mutex::new(rx_ui)),
                    mtx_ui: MAudioSender(Mutex::new(tx_ui)),
                })
            });

            let _ = app.manage(mss);
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application");
    app.run(|_app_handle, event| match event {
        tauri::RunEvent::ExitRequested { api, .. } => {
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
