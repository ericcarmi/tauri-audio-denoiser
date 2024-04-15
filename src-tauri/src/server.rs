use crate::settings::{Colors, Theme};
use crate::types::{AudioParams, StereoControl, UIParams};
use crate::{constants::*, settings::Settings, types::Bpf};
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use redis::Commands;
use rustfft::{num_complex::Complex, FftPlanner};
use serde::Serialize;
use std::fs::File;
use tauri::AppHandle;

// use serde::{Deserialize, Serialize};

// async fn fetch_fft(file_name: String) -> redis::RedisResult<String> {
//     let mut con = get_client_connection().await.unwrap();
//     let r = con.get(file_name + "-fft");

//     r
// }

async fn get_client_connection() -> redis::RedisResult<MultiplexedConnection> {
    let client = redis::Client::open(format!("redis://127.0.0.1:{}/", REDIS_PORT))?;
    let con = client.get_multiplexed_async_connection().await?;

    Ok(con)
}

// #[tauri::command]
// pub async fn get_file_fft(file_name: &str, app_handle: AppHandle) -> Result<Vec<f32>, String> {
//     let p = app_handle
//         .path_resolver()
//         .resolve_resource(ASSETS_PATH)
//         .expect("failed to resolve resource")
//         .into_os_string()
//         .into_string()
//         .unwrap();

//     let filepath = p + "/" + file_name;
//     let handle = tauri::async_runtime::spawn(async move {
//         let r = fetch_fft(filepath);
//         return r;
//     });

//     let r = handle.await;

//     if r.is_ok() {
//         // Ok(r.unwrap().unwrap())
//         let de: Vec<f32> = serde_json::from_str(r.unwrap().unwrap().as_str()).unwrap();
//         Ok(de)
//     } else {
//         let s = r.err().unwrap();
//         Err(s.to_string())
//     }
// }

#[tauri::command]
pub async fn set_file_fft(file_name: &str, app_handle: AppHandle) -> Result<(), String> {
    let p = app_handle
        .path_resolver()
        .resolve_resource(ASSETS_PATH)
        .expect("failed to resolve resource")
        .into_os_string()
        .into_string()
        .unwrap();

    let filepath = p + "/" + file_name;

    let _r = redis_set_file_fft(filepath.as_str()).await;

    Ok(())
}

async fn redis_set_file_fft(file_name: &str) -> redis::RedisResult<()> {
    let p = file_name.to_string() + "-fft";
    let filepath = file_name.to_string();

    let thread = tauri::async_runtime::spawn(async move {
        let file_in = File::open(filepath).unwrap();
        let (head, w) = wav_io::read_from_file(file_in).unwrap();
        let mut buffer = vec![];
        let len = w.len();
        for s in w.clone() {
            let x = s;
            buffer.push(Complex { re: x, im: 0.0f32 })
        }
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(len);

        fft.process(&mut buffer);
        return (
            w,
            buffer[0..len / 2]
                .iter()
                .map(|x| x.norm())
                .collect::<Vec<f32>>(),
        );
    });

    // if thread has error, problem fft or reading file
    // no error means go ahead and set the data in redis
    let r = thread.await;
    let data = r.unwrap().1;

    // let data: String = vec![1, 0].iter().map(|x| x.to_string() + ",").collect();

    // let client = redis::Client::open("redis://127.0.0.1/")?;
    // let mut con = client.get_multiplexed_async_connection().await?;
    let mut con = get_client_connection().await.unwrap();

    // let x = con.set(p, data).await;
    let x = con.set(p, serde_json::to_string(&data).unwrap()).await;
    x
}

async fn redis_save_global_state(bpfs: Vec<Bpf>) -> redis::RedisResult<()> {
    let mut con = get_client_connection().await.unwrap();

    for (i, bpf) in bpfs.iter().enumerate() {
        let a: Result<(), redis::RedisError> = con.set(format!("gain-{}", i + 1), bpf.gain).await;
        if a.is_err() {
            return a;
        }
        let b: Result<(), redis::RedisError> = con.set(format!("freq-{}", i + 1), bpf.freq).await;
        if b.is_err() {
            return b;
        }
        let c: Result<(), redis::RedisError> = con.set(format!("Q-{}", i + 1), bpf.Q).await;
        if c.is_err() {
            return c;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn save_global_state(bpfs: Vec<Bpf>) {
    let _r = redis_save_global_state(bpfs).await;
}

async fn redis_get_global_state() -> redis::RedisResult<Vec<Bpf>> {
    let mut con = get_client_connection().await.unwrap();

    let mut bpfs = vec![];
    for i in 0..5 {
        let gain: Result<f32, redis::RedisError> = con.get(format!("gain-{}", i + 1)).await;
        if gain.is_err() {
            // this isn't good but weird since this needs to turn bpf and redis result
            return Ok(bpfs);
        }
        let freq: Result<f32, redis::RedisError> = con.get(format!("freq-{}", i + 1)).await;
        if freq.is_err() {
            return Ok(bpfs);
        }
        let Q: Result<f32, redis::RedisError> = con.get(format!("Q-{}", i + 1)).await;
        if Q.is_err() {
            return Ok(bpfs);
        }
        bpfs.push(Bpf {
            gain: gain.unwrap(),
            freq: freq.unwrap(),
            Q: Q.unwrap(),
        })
    }

    Ok(bpfs)
}

#[tauri::command]
pub async fn get_global_state() -> Result<Vec<Bpf>, String> {
    if let Ok(bpfs) = redis_get_global_state().await {
        Ok(bpfs)
    } else {
        Err("failed to get global state".to_string())
    }
}

async fn redis_get_channel_state(channel: StereoControl) -> redis::RedisResult<UIParams> {
    let mut con = get_client_connection().await.unwrap();

    let ch = channel.as_str().to_lowercase();

    let mut params = UIParams::new();
    let mut bpfs = vec![];
    for i in 0..5 {
        let gain: Result<f32, redis::RedisError> = con.get(format!("{}_gain-{}", ch, i + 1)).await;
        if gain.is_err() {
            // this isn't good but weird since this needs to turn bpf and redis result
            return Err(gain.err().unwrap());
        }
        let freq: Result<f32, redis::RedisError> = con.get(format!("{}_freq-{}", ch, i + 1)).await;
        if freq.is_err() {
            // return Ok(bpfs);
        }
        let Q: Result<f32, redis::RedisError> = con.get(format!("{}_Q-{}", ch, i + 1)).await;
        if Q.is_err() {
            // return Ok(bpfs);
        }
        bpfs.push(Bpf {
            gain: gain.unwrap(),
            freq: freq.unwrap(),
            Q: Q.unwrap(),
        })
    }
    params.bpfs = bpfs.clone();

    let noise_gain: Result<f32, redis::RedisError> = con.get(format!("{}_noise_gain", ch)).await;
    if noise_gain.is_err() {
        // return Ok(bpfs);
    }
    params.noise_gain = noise_gain.unwrap();
    let output_gain: Result<f32, redis::RedisError> = con.get(format!("{}_output_gain", ch)).await;
    if output_gain.is_err() {
        // return Ok(bpfs);
    }
    params.output_gain = output_gain.unwrap();
    let post_smooth_gain: Result<f32, redis::RedisError> =
        con.get(format!("{}_post_smooth_gain", ch)).await;
    if post_smooth_gain.is_err() {
        // return Ok(bpfs);
    }
    params.post_smooth_gain = post_smooth_gain.unwrap();
    let pre_smooth_gain: Result<f32, redis::RedisError> =
        con.get(format!("{}_pre_smooth_gain", ch)).await;
    if pre_smooth_gain.is_err() {
        // return Ok(bpfs);
    }
    params.pre_smooth_gain = pre_smooth_gain.unwrap();
    let mute: Result<bool, redis::RedisError> = con.get("left_mute").await;
    if mute.is_err() {
        // return Ok(bpfs);
    }
    params.left_mute = mute.unwrap();
    let mute: Result<bool, redis::RedisError> = con.get("right_mute").await;
    if mute.is_err() {
        // return Ok(bpfs);
    }
    params.right_mute = mute.unwrap();

    Ok(params)
}

#[tauri::command]
pub async fn get_channel_state(channel: StereoControl) -> Result<UIParams, String> {
    let ui_params = redis_get_channel_state(channel).await;
    if ui_params.is_ok() {
        Ok(ui_params.unwrap())
    } else {
        ui_params
            .map_err(|e| "error getting left channel state -- ".to_owned() + e.to_string().as_str())
    }
}

async fn redis_get_noise_gain(stereo_control: Option<StereoControl>) -> redis::RedisResult<f32> {
    let mut con = get_client_connection().await.unwrap();

    let gain: Result<f32, redis::RedisError>;
    use StereoControl::*;

    if let Some(st) = stereo_control {
        match st {
            Left => gain = con.get("left_noise_gain").await,
            Right => gain = con.get("right_noise_gain").await,
            Both => gain = con.get("both_noise_gain").await,
        };
    } else {
        gain = con.get("left_noise_gain").await;
    }
    gain
}

#[tauri::command]
pub async fn get_noise_gain(stereo_control: Option<StereoControl>) -> Result<f32, String> {
    if let Ok(gain) = redis_get_noise_gain(stereo_control).await {
        Ok(gain)
    } else {
        Err("error getting noise gain".to_string())
    }
}

async fn redis_get_mute(stereo_control: StereoControl) -> redis::RedisResult<bool> {
    let mut con = get_client_connection().await.unwrap();

    use StereoControl::*;
    let mute: Result<bool, redis::RedisError> = match stereo_control {
        Left => con.get("left_mute").await,
        Right => con.get("right_mute").await,
        Both => con.get("left_mute").await,
    };

    mute
}

#[tauri::command]
pub async fn get_mute(stereo_control: StereoControl) -> Result<bool, String> {
    if let Ok(mute) = redis_get_mute(stereo_control).await {
        Ok(mute)
    } else {
        Err("failed to get noise mute".to_string())
    }
}

async fn redis_get_output_gain(stereo_control: Option<StereoControl>) -> redis::RedisResult<f32> {
    let mut con = get_client_connection().await.unwrap();

    let gain: Result<f32, redis::RedisError>;
    use StereoControl::*;

    if let Some(st) = stereo_control {
        match st {
            Left => gain = con.get("left_output_gain").await,
            Right => gain = con.get("right_output_gain").await,
            Both => gain = con.get("both_output_gain").await,
        };
    } else {
        gain = con.get("left_output_gain").await;
    }
    gain
}

#[tauri::command]
pub async fn get_output_gain(stereo_control: Option<StereoControl>) -> Result<f32, String> {
    if let Ok(gain) = redis_get_output_gain(stereo_control).await {
        Ok(gain)
    } else {
        Err("error getting noise gain".to_string())
    }
}

async fn redis_get_pre_smooth_gain(
    stereo_control: Option<StereoControl>,
) -> redis::RedisResult<f32> {
    let mut con = get_client_connection().await.unwrap();

    let gain: Result<f32, redis::RedisError>;
    use StereoControl::*;

    if let Some(st) = stereo_control {
        match st {
            Left => gain = con.get("left_pre_smooth_gain").await,
            Right => gain = con.get("right_pre_smooth_gain").await,
            Both => gain = con.get("both_pre_smooth_gain").await,
        };
    } else {
        gain = con.get("left_pre_smooth_gain").await;
    }
    gain
}

#[tauri::command]
pub async fn get_pre_smooth_gain(stereo_control: Option<StereoControl>) -> Result<f32, String> {
    if let Ok(gain) = redis_get_pre_smooth_gain(stereo_control).await {
        Ok(gain)
    } else {
        Err("error getting noise gain".to_string())
    }
}

async fn redis_get_post_smooth_gain(
    stereo_control: Option<StereoControl>,
) -> redis::RedisResult<f32> {
    let mut con = get_client_connection().await.unwrap();

    let gain: Result<f32, redis::RedisError>;
    use StereoControl::*;

    if let Some(st) = stereo_control {
        match st {
            Left => gain = con.get("left_post_smooth_gain").await,
            Right => gain = con.get("right_post_smooth_gain").await,
            Both => gain = con.get("both_post_smooth_gain").await,
        };
    } else {
        gain = con.get("left_post_smooth_gain").await;
    }
    gain
}

#[tauri::command]
pub async fn get_post_smooth_gain(stereo_control: Option<StereoControl>) -> Result<f32, String> {
    if let Ok(gain) = redis_get_post_smooth_gain(stereo_control).await {
        Ok(gain)
    } else {
        Err("error getting noise gain".to_string())
    }
}

async fn redis_save_bpf_gain(
    gain: f32,
    index: usize,
    stereo_control: Option<StereoControl>,
) -> redis::RedisResult<()> {
    let mut con = get_client_connection().await.unwrap();
    let a: Result<(), redis::RedisError>;

    if let Some(stereo) = stereo_control {
        use StereoControl::*;
        match stereo {
            Left => {
                a = con.set(format!("left_gain-{}", index), gain).await;
            }
            Right => {
                a = con.set(format!("right_gain-{}", index), gain).await;
            }
            Both => {
                a = con.set(format!("both_gain-{}", index), gain).await;
            }
        }
    } else {
        a = con.set(format!("left_gain-{}", index), gain).await;
    }
    return a;
}

#[tauri::command]
pub async fn save_bpf_gain(gain: f32, index: usize, stereo_control: Option<StereoControl>) {
    let _r = redis_save_bpf_gain(gain, index, stereo_control).await;
}

async fn redis_save_bpf_freq(
    freq: f32,
    index: usize,
    stereo_control: Option<StereoControl>,
) -> redis::RedisResult<()> {
    let mut con = get_client_connection().await.unwrap();
    let a: Result<(), redis::RedisError>;
    if let Some(stereo) = stereo_control {
        use StereoControl::*;
        match stereo {
            Left => {
                a = con.set(format!("left_freq-{}", index), freq).await;
            }
            Right => {
                a = con.set(format!("right_freq-{}", index), freq).await;
            }
            Both => {
                a = con.set(format!("both_freq-{}", index), freq).await;
            }
        }
    } else {
        a = con.set(format!("left_freq-{}", index), freq).await;
    }
    return a;
}

#[tauri::command]
pub async fn save_bpf_freq(freq: f32, index: usize, stereo_control: Option<StereoControl>) {
    let _r = redis_save_bpf_freq(freq, index, stereo_control).await;
}

async fn redis_save_bpf_Q(
    Q: f32,
    index: usize,
    stereo_control: Option<StereoControl>,
) -> redis::RedisResult<()> {
    let mut con = get_client_connection().await.unwrap();
    let a: Result<(), redis::RedisError>;
    let b: Result<(), redis::RedisError>;
    if let Some(stereo) = stereo_control {
        use StereoControl::*;
        match stereo {
            Left => {
                a = con.set(format!("left_Q-{}", index), Q).await;
            }
            Right => {
                a = con.set(format!("right_Q-{}", index), Q).await;
            }
            Both => {
                a = con.set(format!("both_Q-{}", index), Q).await;
            }
        }
    } else {
        a = con.set(format!("left_Q-{}", index), Q).await;
    }
    return a;
}

#[tauri::command]
pub async fn save_bpf_Q(Q: f32, index: usize, stereo_control: Option<StereoControl>) {
    let _r = redis_save_bpf_Q(Q, index, stereo_control).await;
}

async fn redis_get_stereo_control() -> redis::RedisResult<String> {
    let mut con = get_client_connection().await.unwrap();
    let st: Result<String, redis::RedisError> = con.get("stereo_control").await;
    st
}

#[tauri::command]
pub async fn get_stereo_control() -> Result<StereoControl, String> {
    if let Ok(st) = redis_get_stereo_control().await {
        use StereoControl::*;

        let s: StereoControl = match st.as_str() {
            "Left" => Left,
            "Right" => Right,
            "Both" => Both,
            _ => return Err("something wrong with stereo_control in database".to_string()),
        };

        Ok(s)
    } else {
        Err("failed to get stereo control".to_string())
    }
}

async fn redis_save_stereo_control(stereo_control: StereoControl) -> redis::RedisResult<()> {
    let mut con = get_client_connection().await.unwrap();
    let a: Result<(), redis::RedisError> = con.set("stereo_control", stereo_control.as_str()).await;
    return a;
}

#[tauri::command]
pub async fn save_stereo_control(stereo_control: StereoControl) {
    let _r = redis_save_stereo_control(stereo_control).await;
}

async fn redis_save_output_gain(
    gain: f32,
    stereo_control: Option<StereoControl>,
) -> redis::RedisResult<()> {
    let mut con = get_client_connection().await.unwrap();
    let a: Result<(), redis::RedisError>;
    let b: Result<(), redis::RedisError>;
    if let Some(stereo) = stereo_control {
        use StereoControl::*;
        match stereo {
            Left => {
                a = con.set("left_output_gain", gain).await;
            }
            Right => {
                a = con.set("right_output_gain", gain).await;
            }
            Both => {
                a = con.set("both_output_gain", gain).await;
            }
        }
    } else {
        a = con.set("left_output_gain", gain).await;
    }
    return a;
}

#[tauri::command]
pub async fn save_output_gain(gain: f32, stereo_control: Option<StereoControl>) {
    let _r = redis_save_output_gain(gain, stereo_control).await;
}

async fn redis_save_pre_smooth_gain(
    gain: f32,
    stereo_control: Option<StereoControl>,
) -> redis::RedisResult<()> {
    let mut con = get_client_connection().await.unwrap();
    let a: Result<(), redis::RedisError>;
    let b: Result<(), redis::RedisError>;
    if let Some(stereo) = stereo_control {
        use StereoControl::*;
        match stereo {
            Left => {
                a = con.set("left_pre_smooth_gain", gain).await;
            }
            Right => {
                a = con.set("right_pre_smooth_gain", gain).await;
            }
            Both => {
                a = con.set("both_pre_smooth_gain", gain).await;
            }
        }
    } else {
        a = con.set("left_pre_smooth_gain", gain).await;
    }
    return a;
}

#[tauri::command]
pub async fn save_pre_smooth_gain(gain: f32, stereo_control: Option<StereoControl>) {
    let _r = redis_save_pre_smooth_gain(gain, stereo_control).await;
}

async fn redis_save_post_smooth_gain(
    gain: f32,
    stereo_control: Option<StereoControl>,
) -> redis::RedisResult<()> {
    let mut con = get_client_connection().await.unwrap();
    let a: Result<(), redis::RedisError>;
    let b: Result<(), redis::RedisError>;
    if let Some(stereo) = stereo_control {
        use StereoControl::*;
        match stereo {
            Left => {
                a = con.set("left_post_smooth_gain", gain).await;
            }
            Right => {
                a = con.set("right_post_smooth_gain", gain).await;
            }
            Both => {
                a = con.set("both_post_smooth_gain", gain).await;
            }
        }
    } else {
        a = con.set("left_post_smooth_gain", gain).await;
    }
    return a;
}

#[tauri::command]
pub async fn save_post_smooth_gain(gain: f32, stereo_control: Option<StereoControl>) {
    let _r = redis_save_post_smooth_gain(gain, stereo_control).await;
}

async fn redis_save_noise_gain(
    gain: f32,
    stereo_control: Option<StereoControl>,
) -> redis::RedisResult<()> {
    let mut con = get_client_connection().await.unwrap();
    let a: Result<(), redis::RedisError>;
    if let Some(stereo) = stereo_control {
        use StereoControl::*;
        match stereo {
            Left => {
                a = con.set("left_noise_gain", gain).await;
            }
            Right => {
                a = con.set("right_noise_gain", gain).await;
            }
            Both => {
                a = con.set("both_noise_gain", gain).await;
            }
        }
    } else {
        a = con.set("left_noise_gain", gain).await;
    }
    return a;
}

#[tauri::command]
pub async fn save_noise_gain(gain: f32, stereo_control: Option<StereoControl>) {
    let _r = redis_save_noise_gain(gain, stereo_control).await;
}

async fn redis_save_mute(
    mute: bool,
    stereo_control: Option<StereoControl>,
) -> redis::RedisResult<()> {
    let mut con = get_client_connection().await.unwrap();
    let a: Result<(), redis::RedisError>;
    let b: Result<(), redis::RedisError>;
    if let Some(stereo) = stereo_control {
        use StereoControl::*;
        match stereo {
            Left => {
                a = con.set("left_mute", mute).await;
            }
            Right => {
                a = con.set("right_mute", mute).await;
            }
            Both => {
                a = con.set("left_mute", mute).await;
                b = con.set("right_mute", mute).await;
            }
        }
    } else {
        a = con.set("left_mute", mute).await;
    }
    return a;
}

#[tauri::command]
pub async fn save_mute(mute: bool, stereo_control: Option<StereoControl>) {
    let _r = redis_save_mute(mute, stereo_control).await;
}

// for creating the default settings in db
#[tauri::command]
pub async fn init_settings() {
    let settings = Settings::default();
    let _r = redis_save_settings(settings).await;
    for sc in StereoControl::iter() {
        let _r = redis_save_noise_gain(0.0, Some(sc)).await;
        let _r = redis_save_output_gain(0.0, Some(sc)).await;
        let _r = redis_save_pre_smooth_gain(0.0, Some(sc)).await;
        let _r = redis_save_post_smooth_gain(0.0, Some(sc)).await;
        let _r = redis_save_post_smooth_gain(0.0, Some(sc)).await;
        let _r = redis_save_stereo_control(sc).await;
        let _r = redis_save_mute(false, Some(sc)).await;
        let _r = redis_save_mute(false, Some(sc)).await;
        for i in 1..=5 {
            let _r = redis_save_bpf_gain(0.0, i, Some(sc)).await;
            let _r = redis_save_bpf_freq(1000.0, i, Some(sc)).await;
            let _r = redis_save_bpf_Q(1.0, i, Some(sc)).await;

            let _r = redis_save_bpf_gain(0.0, i, Some(sc)).await;
            let _r = redis_save_bpf_freq(1000.0, i, Some(sc)).await;
            let _r = redis_save_bpf_Q(1.0, i, Some(sc)).await;

            let _r = redis_save_bpf_gain(0.0, i, Some(sc)).await;
            let _r = redis_save_bpf_freq(1000.0, i, Some(sc)).await;
            let _r = redis_save_bpf_Q(1.0, i, Some(sc)).await;
        }
    }
}

async fn redis_save_settings(settings: Settings) -> redis::RedisResult<()> {
    let mut con = get_client_connection().await.unwrap();
    let ser = serde_json::to_string(&settings).unwrap();
    let a: Result<(), redis::RedisError> = con.set("settings", ser).await;
    return a;
}

#[tauri::command]
pub async fn save_settings(settings: Settings) {
    let _r = redis_save_settings(settings).await;
}

async fn redis_get_settings() -> redis::RedisResult<String> {
    let mut con = get_client_connection().await.unwrap();

    let sett: Result<String, redis::RedisError> = con.get("settings").await;

    sett
}

#[tauri::command]
pub async fn get_settings() -> Result<Settings, String> {
    if let Ok(sett) = redis_get_settings().await {
        let settings: Settings = serde_json::from_str(sett.as_str()).unwrap();

        Ok(settings)
    } else {
        Err("failed to get settings".to_string())
    }
}

async fn redis_save_theme(theme: Theme) -> redis::RedisResult<()> {
    let mut con = get_client_connection().await.unwrap();
    let ser = serde_json::to_string(&theme).unwrap();
    let a: Result<(), redis::RedisError> = con.set(format!("theme-{}", theme.as_str()), ser).await;
    return a;
}

#[tauri::command]
pub async fn save_theme(theme: Theme) {
    let _r = redis_save_theme(theme).await;
}

async fn redis_get_theme() -> redis::RedisResult<String> {
    let mut con = get_client_connection().await.unwrap();

    let sett: Result<String, redis::RedisError> = con.get("theme").await;

    sett
}

#[tauri::command]
pub async fn get_theme() -> Result<Theme, String> {
    if let Ok(sett) = redis_get_theme().await {
        let theme: Theme = serde_json::from_str(sett.as_str()).unwrap();

        Ok(theme)
    } else {
        Err("failed to get theme".to_string())
    }
}

#[tauri::command]
pub fn get_theme_colors(name: &str) -> Result<Colors, String> {
    let theme = Theme::RGB;
    match name {
        "RGB" => Ok(theme.rgb()),
        "CYM" => Ok(theme.cym()),
        "POG" => Ok(theme.pog()),
        // "CUSTOM" => theme.custom(),
        _ => Err("invalid theme name".to_string()),
    }
}
