use wavers::Wav;

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
        let w: Vec<f32> = Wav::from_path(filepath).unwrap().read().unwrap().to_vec();
        return w;
    });

    if let Ok(r) = thread.await {
        time_data = r;
    }

    Ok(time_data)
}
