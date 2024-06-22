use crate::{
    constants::NUM_FILTERS,
    settings::{ComponentColors, Settings, Theme},
    types::{StereoChoice, UIFilters, UIParams, BPF},
};
use rusqlite::{Connection, Result};
use tauri::AppHandle;
pub const DB_FILE_NAME: &'static str = "/db.sqlite";

pub fn query_theme_name(s: String) -> Result<Theme, rusqlite::Error> {
    let conn = Connection::open(s + DB_FILE_NAME)?;
    let mut stmt = conn.prepare("SELECT theme FROM settings")?;
    let mut rows = stmt.query([])?;
    let mut theme: Theme = Theme::RGB;

    while let Some(row) = rows.next()? {
        theme = row.get(0)?;
    }
    Ok(theme)
}

#[tauri::command]
pub fn sql_theme_name(app_handle: AppHandle) -> Result<Theme, String> {
    let p = app_handle
        .path_resolver()
        .resource_dir()
        .expect("failed to resolve resource")
        .into_os_string()
        .into_string()
        .unwrap();

    let q = query_theme_name(p);

    if q.is_ok() {
        return Ok(q.unwrap());
    } else {
        return q.map_err(|e| e.to_string());
    }
}

pub fn query_theme(s: String, theme: Theme) -> Result<ComponentColors, rusqlite::Error> {
    let conn = Connection::open(s + DB_FILE_NAME)?;
    let mut stmt = conn.prepare(
        format!(
            "SELECT * FROM COMPONENTCOLORS WHERE name='{}'",
            theme.as_str()
        )
        .as_str(),
    )?;

    let colors_iter = stmt.query_map([], |row| {
        Ok(ComponentColors {
            rotary_tick: row.get(2)?,
            rotary_hover: row.get(3)?,
            slider_background: row.get(4)?,
            slider_hover: row.get(5)?,
            slider_border: row.get(6)?,
            slider_active: row.get(7)?,
            slider_indicator: row.get(8)?,
            plot_main: row.get(9)?,
            plot_single_filter: row.get(10)?,
            plot_total_curve: row.get(11)?,
            plot_filter_hover: row.get(12)?,
            app_background: row.get(13)?,
            app_text: row.get(14)?,
        })
    })?;

    for colors in colors_iter {
        if let Ok(c) = colors {
            return Ok(c);
        }
    }
    return Err(rusqlite::Error::InvalidQuery);
}

#[tauri::command]
pub fn sql_theme(theme: Theme, app_handle: AppHandle) -> Result<ComponentColors, String> {
    let p = app_handle
        .path_resolver()
        .resource_dir()
        .expect("failed to resolve resource")
        .into_os_string()
        .into_string()
        .unwrap();

    let q = query_theme(p, theme);

    if q.is_ok() {
        return Ok(q.unwrap());
    } else {
        return q.map_err(|e| e.to_string());
    }
}

pub fn query_settings(s: String) -> Result<Settings, rusqlite::Error> {
    let conn = Connection::open(s + DB_FILE_NAME)?;
    let mut stmt = conn.prepare("SELECT * FROM SETTINGS")?;
    let settings_iter = stmt.query_map([], |row| {
        Ok(Settings {
            id: row.get(0)?,
            plot_scale: row.get(1)?,
            theme: row.get(2)?,
            draw_freq_axis: row.get(3)?,
            draw_fft_amp_axis: row.get(4)?,
            draw_filter_amp_axis: row.get(5)?,
        })
    })?;

    for settings in settings_iter {
        if let Ok(sett) = settings {
            return Ok(sett);
        }
    }
    return Err(rusqlite::Error::InvalidQuery);
}

pub fn query_ui_params(
    stereo_choice: StereoChoice,
    s: String,
) -> Result<UIParams, rusqlite::Error> {
    let conn = Connection::open(s + DB_FILE_NAME)?;
    let chan = stereo_choice.as_str().to_lowercase();
    let mut stmt =
        conn.prepare(format!("SELECT * FROM UI_PARAMS WHERE stereo_choice='{}'", chan).as_str())?;
    let control_iter = stmt.query_map([], |row| {
        Ok(UIParams {
            // id: row.get(0)?,
            clean: row.get(2)?,
            left_mute: row.get(3)?,
            right_mute: row.get(4)?,
            output_gain: row.get(5)?,
            noise_gain: row.get(6)?,
            pre_smooth_gain: row.get(7)?,
            post_smooth_gain: row.get(8)?,
            ..Default::default()
        })
    })?;

    for control in control_iter {
        if let Ok(sett) = control {
            return Ok(sett);
        }
    }
    return Err(rusqlite::Error::InvalidQuery);
}

#[tauri::command]
pub fn sql_ui_params(
    stereo_choice: StereoChoice,
    app_handle: AppHandle,
) -> Result<UIParams, String> {
    let p = app_handle
        .path_resolver()
        .resource_dir()
        .expect("failed to resolve resource")
        .into_os_string()
        .into_string()
        .unwrap();

    let q = query_ui_params(stereo_choice, p);

    if q.is_ok() {
        return Ok(q.unwrap());
    } else {
        return q.map_err(|e| e.to_string());
    }
}

#[tauri::command]
pub fn sql_settings(app_handle: AppHandle) -> Result<Settings, String> {
    let p = app_handle
        .path_resolver()
        .resource_dir()
        .expect("failed to resolve resource")
        .into_os_string()
        .into_string()
        .unwrap();

    let q = query_settings(p);

    if q.is_ok() {
        return Ok(q.unwrap());
    } else {
        return q.map_err(|e| e.to_string());
    }
}

pub fn query_filter_bank(
    stereo_choice: StereoChoice,
    s: String,
) -> Result<UIFilters, rusqlite::Error> {
    let conn = Connection::open(s + DB_FILE_NAME)?;
    let chan = stereo_choice.as_str().to_lowercase();
    let mut stmt =
        conn.prepare(format!("SELECT * FROM FILTERBANK WHERE stereo_choice='{}'", chan).as_str())?;
    let control_iter = stmt.query_map([], |row| {
        let offset = 2;
        let mut ui_filters = UIFilters::new();
        for i in 0..NUM_FILTERS {
            ui_filters.bank[i] = BPF {
                gain: row.get(offset + 3 * i)?,
                freq: row.get(offset + 3 * i + 1)?,
                Q: row.get(offset + 3 * i + 2)?,
            };
        }
        Ok(ui_filters)
    })?;

    for control in control_iter {
        if let Ok(sett) = control {
            return Ok(sett);
        }
    }
    return Err(rusqlite::Error::InvalidQuery);
}

#[tauri::command]
pub fn sql_filter_bank(
    stereo_choice: StereoChoice,
    app_handle: AppHandle,
) -> Result<UIFilters, String> {
    let p = app_handle
        .path_resolver()
        .resource_dir()
        .expect("failed to resolve resource")
        .into_os_string()
        .into_string()
        .unwrap();

    let q = query_filter_bank(stereo_choice, p);

    if q.is_ok() {
        return Ok(q.unwrap());
    } else {
        return q.map_err(|e| e.to_string());
    }
}

/// update params AND filter bank, filter bank is part of ui params but treated separately because of how it isn't iterated over and has its own table
pub fn update_ui_params(
    stereo_choice: StereoChoice,
    ui_params: UIParams,
    s: String,
) -> Result<(), rusqlite::Error> {
    let conn = Connection::open(s + DB_FILE_NAME)?;
    let st = stereo_choice.as_str().to_lowercase();

    let mut filter_string = "".to_string();
    let bank = ui_params.filters.bank;
    for i in 0..NUM_FILTERS {
        filter_string += format!(
            " bpf_gain_{} = {}, bpf_freq_{} = {}, bpf_Q_{} = {},",
            i, bank[i].gain, i, bank[i].freq, i, bank[i].Q
        )
        .as_str()
    }
    filter_string.pop();

    let mut q = format!("UPDATE UI_PARAMS SET clean={}, left_mute={}, right_mute={}, output_gain={}, noise_gain={}, pre_smooth_gain={}, post_smooth_gain={}  WHERE stereo_choice='{}';
        UPDATE FILTERBANK SET          
        ",ui_params.clean, ui_params.left_mute, ui_params.right_mute, ui_params.output_gain, ui_params.noise_gain, ui_params.pre_smooth_gain, ui_params.post_smooth_gain, st);

    let end_string = format!(" WHERE stereo_choice='{}'", st);
    q += filter_string.as_str();
    q += end_string.as_str();

    conn.execute_batch(q.as_str())?;

    Ok(())
}

#[tauri::command]
pub fn sql_update_ui_params(
    stereo_choice: StereoChoice,
    ui_params: UIParams,
    app_handle: AppHandle,
) -> Result<(), String> {
    let p = app_handle
        .path_resolver()
        .resource_dir()
        .expect("failed to resolve resource")
        .into_os_string()
        .into_string()
        .unwrap();

    let q = update_ui_params(stereo_choice, ui_params, p);

    if q.is_ok() {
        return Ok(q.unwrap());
    } else {
        return q.map_err(|e| e.to_string());
    }
}

pub fn update_filter_bank(
    stereo_choice: StereoChoice,
    bpf: BPF,
    index: usize,
    s: String,
) -> Result<(), rusqlite::Error> {
    let conn = Connection::open(s + DB_FILE_NAME)?;
    let st = stereo_choice.as_str().to_lowercase();
    let q = format!(
        "UPDATE FILTERBANK SET 
        bpf_gain_{} = {}, bpf_freq_{} = {}, bpf_q_{} = {}  
        WHERE stereo_choice='{}';
        ",
        index, bpf.gain, index, bpf.freq, index, bpf.Q, st
    );

    conn.execute_batch(q.as_str())?;

    Ok(())
}

#[tauri::command]
pub fn sql_update_filter_bank(
    stereo_choice: StereoChoice,
    bpf: BPF,
    index: usize,
    app_handle: AppHandle,
) -> Result<(), String> {
    let p = app_handle
        .path_resolver()
        .resource_dir()
        .expect("failed to resolve resource")
        .into_os_string()
        .into_string()
        .unwrap();

    let q = update_filter_bank(stereo_choice, bpf, index, p);

    if q.is_ok() {
        return Ok(q.unwrap());
    } else {
        return q.map_err(|e| e.to_string());
    }
}

pub fn update_output_gain(
    stereo_choice: StereoChoice,
    output_gain: f32,
    s: String,
) -> Result<(), rusqlite::Error> {
    let conn = Connection::open(s + DB_FILE_NAME)?;
    let st = stereo_choice.as_str().to_lowercase();
    let q = format!(
        "UPDATE UI_PARAMS SET output_gain={} WHERE stereo_choice='{}';
        ",
        output_gain, st
    );

    conn.execute_batch(q.as_str())?;

    Ok(())
}

#[tauri::command]
pub fn sql_update_output_gain(
    stereo_choice: StereoChoice,
    output_gain: f32,
    app_handle: AppHandle,
) -> Result<(), String> {
    let p = app_handle
        .path_resolver()
        .resource_dir()
        .expect("failed to resolve resource")
        .into_os_string()
        .into_string()
        .unwrap();

    let q = update_output_gain(stereo_choice, output_gain, p);

    if q.is_ok() {
        return Ok(q.unwrap());
    } else {
        return q.map_err(|e| e.to_string());
    }
}

pub fn update_noise_gain(
    stereo_choice: StereoChoice,
    noise_gain: f32,
    s: String,
) -> Result<(), rusqlite::Error> {
    let conn = Connection::open(s + DB_FILE_NAME)?;
    let st = stereo_choice.as_str().to_lowercase();
    let q = format!(
        "UPDATE UI_PARAMS SET noise_gain={} WHERE stereo_choice='{}';
        ",
        noise_gain, st
    );

    conn.execute_batch(q.as_str())?;

    Ok(())
}

#[tauri::command]
pub fn sql_update_noise_gain(
    stereo_choice: StereoChoice,
    noise_gain: f32,
    app_handle: AppHandle,
) -> Result<(), String> {
    let p = app_handle
        .path_resolver()
        .resource_dir()
        .expect("failed to resolve resource")
        .into_os_string()
        .into_string()
        .unwrap();

    let q = update_noise_gain(stereo_choice, noise_gain, p);

    if q.is_ok() {
        return Ok(q.unwrap());
    } else {
        return q.map_err(|e| e.to_string());
    }
}

pub fn update_pre_smooth_gain(
    stereo_choice: StereoChoice,
    pre_smooth_gain: f32,
    s: String,
) -> Result<(), rusqlite::Error> {
    let conn = Connection::open(s + DB_FILE_NAME)?;
    let st = stereo_choice.as_str().to_lowercase();
    let q = format!(
        "UPDATE UI_PARAMS SET pre_smooth_gain={} WHERE stereo_choice='{}';
        ",
        pre_smooth_gain, st
    );

    conn.execute_batch(q.as_str())?;

    Ok(())
}

#[tauri::command]
pub fn sql_update_pre_smooth_gain(
    stereo_choice: StereoChoice,
    pre_smooth_gain: f32,
    app_handle: AppHandle,
) -> Result<(), String> {
    let p = app_handle
        .path_resolver()
        .resource_dir()
        .expect("failed to resolve resource")
        .into_os_string()
        .into_string()
        .unwrap();

    let q = update_pre_smooth_gain(stereo_choice, pre_smooth_gain, p);

    if q.is_ok() {
        return Ok(q.unwrap());
    } else {
        return q.map_err(|e| e.to_string());
    }
}

pub fn update_post_smooth_gain(
    stereo_choice: StereoChoice,
    post_smooth_gain: f32,
    s: String,
) -> Result<(), rusqlite::Error> {
    let conn = Connection::open(s + DB_FILE_NAME)?;
    let st = stereo_choice.as_str().to_lowercase();
    let q = format!(
        "UPDATE UI_PARAMS SET post_smooth_gain={} WHERE stereo_choice='{}';
        ",
        post_smooth_gain, st
    );

    conn.execute_batch(q.as_str())?;

    Ok(())
}

#[tauri::command]
pub fn sql_update_post_smooth_gain(
    stereo_choice: StereoChoice,
    post_smooth_gain: f32,
    app_handle: AppHandle,
) -> Result<(), String> {
    let p = app_handle
        .path_resolver()
        .resource_dir()
        .expect("failed to resolve resource")
        .into_os_string()
        .into_string()
        .unwrap();

    let q = update_post_smooth_gain(stereo_choice, post_smooth_gain, p);

    if q.is_ok() {
        return Ok(q.unwrap());
    } else {
        return q.map_err(|e| e.to_string());
    }
}

pub fn update_clean(
    stereo_choice: StereoChoice,
    clean: bool,
    s: String,
) -> Result<(), rusqlite::Error> {
    let conn = Connection::open(s + DB_FILE_NAME)?;
    let st = stereo_choice.as_str().to_lowercase();
    let q = format!(
        "UPDATE UI_PARAMS SET clean={} WHERE stereo_choice='{}';
        ",
        clean, st
    );

    conn.execute_batch(q.as_str())?;

    Ok(())
}

#[tauri::command]
pub fn sql_update_clean(
    stereo_choice: StereoChoice,
    clean: bool,
    app_handle: AppHandle,
) -> Result<(), String> {
    let p = app_handle
        .path_resolver()
        .resource_dir()
        .expect("failed to resolve resource")
        .into_os_string()
        .into_string()
        .unwrap();

    let q = update_clean(stereo_choice, clean, p);

    if q.is_ok() {
        return Ok(q.unwrap());
    } else {
        return q.map_err(|e| e.to_string());
    }
}

pub fn update_left_mute(
    stereo_choice: StereoChoice,
    left_mute: bool,
    s: String,
) -> Result<(), rusqlite::Error> {
    let conn = Connection::open(s + DB_FILE_NAME)?;
    let st = stereo_choice.as_str().to_lowercase();
    let q = format!(
        "UPDATE UI_PARAMS SET left_mute={} WHERE stereo_choice='{}';
        ",
        left_mute, st
    );

    conn.execute_batch(q.as_str())?;

    Ok(())
}

#[tauri::command]
pub fn sql_update_left_mute(
    stereo_choice: StereoChoice,
    left_mute: bool,
    app_handle: AppHandle,
) -> Result<(), String> {
    let p = app_handle
        .path_resolver()
        .resource_dir()
        .expect("failed to resolve resource")
        .into_os_string()
        .into_string()
        .unwrap();

    let q = update_left_mute(stereo_choice, left_mute, p);

    if q.is_ok() {
        return Ok(q.unwrap());
    } else {
        return q.map_err(|e| e.to_string());
    }
}

pub fn update_right_mute(
    stereo_choice: StereoChoice,
    right_mute: bool,
    s: String,
) -> Result<(), rusqlite::Error> {
    let conn = Connection::open(s + DB_FILE_NAME)?;
    let st = stereo_choice.as_str().to_lowercase();
    let q = format!(
        "UPDATE UI_PARAMS SET right_mute={} WHERE stereo_choice='{}';
        ",
        right_mute, st
    );

    conn.execute_batch(q.as_str())?;

    Ok(())
}

#[tauri::command]
pub fn sql_update_right_mute(
    stereo_choice: StereoChoice,
    right_mute: bool,
    app_handle: AppHandle,
) -> Result<(), String> {
    let p = app_handle
        .path_resolver()
        .resource_dir()
        .expect("failed to resolve resource")
        .into_os_string()
        .into_string()
        .unwrap();

    let q = update_right_mute(stereo_choice, right_mute, p);

    if q.is_ok() {
        return Ok(q.unwrap());
    } else {
        return q.map_err(|e| e.to_string());
    }
}
