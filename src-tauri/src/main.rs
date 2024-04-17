// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]
use cpal::traits::StreamTrait;
use rusqlite::Connection;
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    sync::Mutex,
};
use tauri::{AppHandle, Manager, State};
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
mod messages;
use messages::*;
mod file_io;
mod settings;
use file_io::*;
use log::info;
mod sql;
use simplelog::*;
use sql::*;
use tauri::api::process::Command as CMD;

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
            set_file_fft,
            get_fft_plot_data,
            get_is_stereo,
            save_global_state,
            get_global_state,
            get_channel_state,
            save_bpf_gain,
            save_bpf_freq,
            save_bpf_Q,
            update_output_gain,
            update_noise_gain,
            save_output_gain,
            save_noise_gain,
            get_noise_gain,
            get_pre_smooth_gain,
            get_post_smooth_gain,
            get_stereo_control,
            save_stereo_control,
            save_pre_smooth_gain,
            save_post_smooth_gain,
            get_output_gain,
            update_pre_smooth_gain,
            update_post_smooth_gain,
            get_settings,
            save_settings,
            init_settings,
            get_theme_colors,
            update_file_path,
            update_mute,
            get_mute,
            save_mute,
            init_audio_params_from_server,
            process_export,
            get_audioui_message,
            sql_create,
            sql_query,
            sql_update,
        ])
        .setup(|app| {
            let mainwindow = app.get_window("main").unwrap();
            let _ = mainwindow.set_always_on_top(true);
            let app_handle = app.app_handle();

            let p = app_handle
                .path_resolver()
                .resource_dir()
                .expect("failed to resolve resource")
                .into_os_string()
                .into_string()
                .unwrap();

            // let r = sql::create();
            // println!("{:?}", r);
            // let r = sql::query(c);
            // println!("{:?}", r);

            let _w = WriteLogger::init(
                LevelFilter::Info,
                Config::default(),
                File::create(p.to_owned() + "/text.log").unwrap(),
            );

            let m = mainwindow.available_monitors();
            start_server(app_handle.clone());
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
            // println!("wtf stop");

            stop_server();
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

fn start_server(app_handle: AppHandle) {
    let p = app_handle
        .path_resolver()
        .resource_dir()
        .expect("failed to resolve resource")
        .into_os_string()
        .into_string()
        .unwrap();

    // println!("{:?}", p);

    let ping = CMD::new_sidecar("redis-cli")
        .expect("failed to stop redis-server")
        .args(["-p", REDIS_PORT, "ping"])
        .output()
        .expect("Failed to spawn sidecar");
    let conf_path = p.clone() + "/redis.conf";

    if let Ok(mut conf) = OpenOptions::new()
        .read(true)
        .write(true)
        .open(conf_path.clone())
    {
        let mut contents = String::new();
        conf.read_to_string(&mut contents).unwrap();
        if !contents.contains("dir ") {
            let r = writeln!(&mut conf, "dir {}", p);
            // info!("{:?}", r);
            // info!("{:?}", contents);
        }
    }

    if !ping.stdout.contains("PONG") {
        let child = CMD::new_sidecar("redis-server")
            .expect("failed to start redis-server")
            .args([conf_path.as_str()])
            .output()
            .expect("Failed to spawn sidecar");
        // println!("server {:?}", child);
        let child = CMD::new_sidecar("redis-cli")
            .expect("failed to start redis-server")
            .args(["-p", REDIS_PORT, "config", "get", "dir"])
            .output()
            .expect("Failed to spawn sidecar");

        // info!("{:?}", child);
        // info!("{:?}", p);
    }
}

fn stop_server() {
    let child = CMD::new_sidecar("redis-cli")
        .expect("failed to stop redis-server")
        .args(["-h", "127.0.0.1", "-p", REDIS_PORT, "shutdown"])
        .output()
        .expect("Failed to spawn sidecar");
    // println!("stop the server dammit{:?}", child);
}
