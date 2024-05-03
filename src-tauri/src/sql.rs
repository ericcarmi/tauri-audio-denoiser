use crate::{
    settings::{Colors, PlotScale, Settings, Theme, GRAY100, GRAY200, GREEN, LIGHTPURPLE, PURPLE},
    types::{AudioParams, StereoChoice, StereoParams, UIFilterBank, UIParams, BPF},
};
use rusqlite::{Connection, Result};
use tauri::AppHandle;
pub const DB_FILE_NAME: &'static str = "/db.sqlite";

pub fn update(s: String) -> Result<()> {
    let conn = Connection::open(s + DB_FILE_NAME)?;
    let me = Settings {
        plot_scale: PlotScale::Linear,
        theme: Theme::RGB,
        draw_freq_axis: true,
        ..Default::default()
    };
    conn.execute(
        "INSERT INTO settings (plot_scale, theme, draw_freq_axis) VALUES (?1, ?2, ?3)",
        (&me.plot_scale, &me.theme, &me.draw_freq_axis),
    )?;
    Ok(())
}

#[tauri::command]
pub fn sql_update(app_handle: AppHandle) -> Result<(), String> {
    let p = app_handle
        .path_resolver()
        .resource_dir()
        .expect("failed to resolve resource")
        .into_os_string()
        .into_string()
        .unwrap();

    if let Err(e) = update(p) {
        return Err(e.to_string());
    }
    Ok(())
}

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

pub fn query_theme(s: String, theme: Theme) -> Result<Colors, rusqlite::Error> {
    let conn = Connection::open(s + DB_FILE_NAME)?;
    let mut stmt =
        conn.prepare(format!("SELECT * FROM themes WHERE name='{}'", theme.as_str()).as_str())?;

    let colors_iter = stmt.query_map([], |row| {
        Ok(Colors {
            rotary_tick: row.get(2)?,
            rotary_hover: row.get(3)?,
            slider_hover: row.get(4)?,
            slider_border: row.get(5)?,
            slider_active: row.get(6)?,
            slider_indicator: row.get(7)?,
            plot_main: row.get(8)?,
            plot_single_filter: row.get(9)?,
            plot_total_curve: row.get(10)?,
            plot_filter_hover: row.get(11)?,
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
pub fn sql_theme(theme: Theme, app_handle: AppHandle) -> Result<Colors, String> {
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
            // output_gain: row.get()
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
) -> Result<UIFilterBank, rusqlite::Error> {
    let conn = Connection::open(s + DB_FILE_NAME)?;
    let chan = stereo_choice.as_str().to_lowercase();
    let mut stmt =
        conn.prepare(format!("SELECT * FROM FILTERBANK WHERE stereo_choice='{}'", chan).as_str())?;
    let control_iter = stmt.query_map([], |row| {
        let offset = 1;
        Ok(UIFilterBank {
            // id: row.get(0)?,
            bp1: BPF {
                gain: row.get(offset + 1)?,
                freq: row.get(offset + 2)?,
                Q: row.get(offset + 3)?,
            },
            bp2: BPF {
                gain: row.get(offset + 4)?,
                freq: row.get(offset + 5)?,
                Q: row.get(offset + 6)?,
            },
            bp3: BPF {
                gain: row.get(offset + 7)?,
                freq: row.get(offset + 8)?,
                Q: row.get(offset + 9)?,
            },
            bp4: BPF {
                gain: row.get(offset + 10)?,
                freq: row.get(offset + 11)?,
                Q: row.get(offset + 12)?,
            },
            bp5: BPF {
                gain: row.get(offset + 13)?,
                freq: row.get(offset + 14)?,
                Q: row.get(offset + 15)?,
            },
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
pub fn sql_filter_bank(
    stereo_choice: StereoChoice,
    app_handle: AppHandle,
) -> Result<UIFilterBank, String> {
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
