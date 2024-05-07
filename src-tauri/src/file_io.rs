use std::{fs::File, path::PathBuf};

use crate::constants::ASSETS_PATH;
#[tauri::command]
pub async fn get_time_data(
    path: &str,
    from_assets: Option<bool>,
    app_handle: tauri::AppHandle,
) -> Result<Vec<f32>, &str> {
    let mut time_data = vec![];

    if from_assets.is_some() {
        let p = app_handle
            .path_resolver()
            .resolve_resource(ASSETS_PATH)
            .expect("failed to resolve resource")
            .into_os_string()
            .into_string()
            .unwrap();

        let filepath = p + "/" + path;

        let thread = tauri::async_runtime::spawn(async move {
            if let Ok(f) = File::open(filepath) {
                let (_head, samples) = wav_io::read_from_file(f).unwrap();

                return samples.iter().step_by(16).cloned().collect::<Vec<f32>>();
            }
            vec![]
        });

        if let Ok(r) = thread.await {
            time_data = r;
        }
    } else {
        let filepath = path.to_owned();

        let thread = tauri::async_runtime::spawn(async move {
            if let Ok(f) = File::open(PathBuf::from(filepath)) {
                let (_head, samples) = wav_io::read_from_file(f).unwrap();

                return samples.iter().step_by(16).cloned().collect::<Vec<f32>>();
            }
            vec![]
        });

        if let Ok(r) = thread.await {
            time_data = r;
        }
    }
    if time_data.is_empty() {
        return Err("failed to get time data");
    }

    Ok(time_data)
}
