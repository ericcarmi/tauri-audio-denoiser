// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]
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
            update_filters,
            update_time,
            update_clean,
            get_stft_data,
            get_time_onefft,
            get_time_data,
            get_fft_plot_data,
            update_output_gain,
            update_noise_gain,
            update_pre_smooth_gain,
            update_post_smooth_gain,
            update_file_path,
            update_mute,
            init_audio_params_from_server,
            process_export,
            get_audioui_message,
            sql_theme,
            sql_theme_name,
            sql_settings,
            sql_update,
            sql_channel_params,
        ])
        .setup(|app| {
            let mainwindow = app.get_window("main").unwrap();
            // let _ = mainwindow.set_always_on_top(true);
            let app_handle = app.app_handle();

            let p = app_handle
                .path_resolver()
                .resource_dir()
                .expect("failed to resolve resource")
                .into_os_string()
                .into_string()
                .unwrap();

            let _w = WriteLogger::init(
                LevelFilter::Info,
                Config::default(),
                File::create(p.to_owned() + "/text.log").unwrap(),
            );

            let m = mainwindow.available_monitors();
            let _ = mainwindow.set_position(*m.unwrap()[0].position());

            let mss = MStreamSend({
                let (tx_ui, rx_ui) = tauri::async_runtime::channel::<AudioUIMessage>(2);
                let (stream, tx) = setup_stream(tx_ui.clone(), app_handle, None).unwrap();
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

#[tauri::command]
fn get_fft_plot_data(streamsend: State<MStreamSend>) -> Result<AudioUIMessage, String> {
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

#[tauri::command]
fn get_audioui_message(streamsend: State<MStreamSend>) -> Result<AudioUIMessage, String> {
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
