use std::fs::File;

use rustfft::num_complex::Complex;
use tauri::State;

use crate::types::{MStreamSend, Message, IIR2};

// pub const TEST_FILE_PATH: &str = "assets/chirp.wav";
// pub const TEST_FILE_PATH: &str = "assets/440-7040-whitenoise.wav";
// pub const TEST_FILE_PATH: &str = "assets/440-whitenoise.wav";
pub const TEST_FILE_PATH: &str = "assets/reisman.wav";
pub const ASSETS_PATH: &str = "assets/";

pub const CZERO: Complex<f32> = Complex { re: 0.0, im: 0.0 };
pub fn czerov(n: usize) -> Vec<Complex<f32>> {
    vec![Complex { re: 0.0, im: 0.0 }; n]
}

#[tauri::command]
pub async fn get_time_data(path: &str, app_handle: tauri::AppHandle) -> Result<Vec<f32>, &str> {
    let mut time_data = vec![];

    let p = app_handle
        .path_resolver()
        .resolve_resource(ASSETS_PATH)
        .expect("failed to resolve resource")
        .into_os_string()
        .into_string()
        .unwrap();

    let filepath = p + "/" + path;
    println!("{:?}", filepath);

    let thread = tauri::async_runtime::spawn(async move {
        let file_in = File::open(filepath).unwrap();
        let (_head, samples) = wav_io::read_from_file(file_in).unwrap();

        return samples.iter().step_by(16).cloned().collect::<Vec<f32>>();
    });

    if let Ok(r) = thread.await {
        time_data = r;
    }

    Ok(time_data)
}

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
            time: None,
            clean: None,
            bp1,
            bp2,
            bp3,
            bp4,
            bp5,
            bypass: None,
            output_gain: None,
            noise_gain: None,
            pre_smooth_gain: None,
            noise_variance: None,
            post_smooth_gain: None,
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
            clean: None,
            bp1: None,
            bp2: None,
            bp3: None,
            bp4: None,
            bp5: None,
            bypass: None,
            output_gain: None,
            noise_gain: None,
            pre_smooth_gain: None,
            noise_variance: None,
            post_smooth_gain: None,
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
            time: None,
            clean: Some(clean),
            bp1: None,
            bp2: None,
            bp3: None,
            bp4: None,
            bp5: None,
            bypass: None,
            output_gain: None,
            noise_gain: None,
            pre_smooth_gain: None,
            noise_variance: None,
            post_smooth_gain: None,
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
            time: None,
            clean: None,
            bp1: None,
            bp2: None,
            bp3: None,
            bp4: None,
            bp5: None,
            bypass: Some(bp),
            output_gain: None,
            noise_gain: None,
            pre_smooth_gain: None,
            noise_variance: None,
            post_smooth_gain: None,
        });
}

#[tauri::command]
pub fn update_output_gain(gain: f32, streamsend: State<MStreamSend>) {
    let _ = streamsend
        .0
        .lock()
        .unwrap()
        .msender
        .0
        .lock()
        .unwrap()
        .try_send(Message {
            time: None,
            clean: None,
            bp1: None,
            bp2: None,
            bp3: None,
            bp4: None,
            bp5: None,
            bypass: None,
            output_gain: Some(gain),
            noise_gain: None,
            pre_smooth_gain: None,
            noise_variance: None,
            post_smooth_gain: None,
        });
}

#[tauri::command]
pub fn update_noise_gain(gain: f32, streamsend: State<MStreamSend>) {
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
            time: None,
            clean: None,
            bp1: None,
            bp2: None,
            bp3: None,
            bp4: None,
            bp5: None,
            bypass: None,
            output_gain: None,
            noise_gain: Some(g),
            pre_smooth_gain: None,
            noise_variance: None,
            post_smooth_gain: None,
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
            time: None,
            clean: None,
            bp1: None,
            bp2: None,
            bp3: None,
            bp4: None,
            bp5: None,
            bypass: None,
            output_gain: None,
            noise_gain: None,
            pre_smooth_gain: Some(gain),
            noise_variance: None,
            post_smooth_gain: None,
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
            time: None,
            clean: None,
            bp1: None,
            bp2: None,
            bp3: None,
            bp4: None,
            bp5: None,
            bypass: None,
            output_gain: None,
            noise_gain: None,
            pre_smooth_gain: None,
            noise_variance: None,
            post_smooth_gain: Some(gain),
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
            time: None,
            clean: None,
            bp1: None,
            bp2: None,
            bp3: None,
            bp4: None,
            bp5: None,
            bypass: None,
            output_gain: None,
            noise_gain: None,
            pre_smooth_gain: None,
            noise_variance: Some(gain),
            post_smooth_gain: None,
        });
}
