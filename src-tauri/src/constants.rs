use tauri::State;
use wavers::Wav;

use crate::types::{MStreamSend, Message, IIR2};

// pub const TEST_FILE_PATH: &str = "assets/chirp.wav";
pub const TEST_FILE_PATH: &str = "assets/test-file.wav";
pub const ASSETS_PATH: &str = "assets/";

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

    let thread = tauri::async_runtime::spawn(async move {
        let w: Vec<f32> = Wav::from_path(filepath)
            .unwrap()
            .read()
            .unwrap()
            .iter()
            .step_by(16)
            .cloned()
            .collect();
        return w;
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
        });
}
