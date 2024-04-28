use crate::settings::{
    Colors, PlotScale, Settings, Theme, GRAY100, GRAY200, GREEN, LIGHTPURPLE, PURPLE,
};
use rusqlite::{Connection, Result};
use tauri::AppHandle;
pub const DB_FILE_NAME: &'static str = "/db.sqlite";

// to avoid requiring sqlite3 being installed on computer, don't run script...which is also a bash script, just another thing to copy tho...
pub fn create(s: String) -> Result<()> {
    let conn = Connection::open(s + DB_FILE_NAME)?;
    conn.execute(
        "CREATE TABLE SETTINGS (
            id    INTEGER PRIMARY KEY,
            plot_scale  TEXT NOT NULL,
            theme  TEXT NOT NULL,
            draw_freq_axis BOOLEAN,
            draw_fft_amp_axis BOOLEAN,
            draw_filter_amp_axis BOOLEAN,
        )",
        (), // empty list of parameters.
    )?;

    conn.execute("insert into settings (plot_scale, theme, draw_freq_axis, draw_fft_amp_axis, draw_filter_amp_axis) values ('Log', 'RGB', true, true, true)", ())?;

    conn.execute(
        "CREATE TABLE THEMES(id INTEGER PRIMARY KEY, name TEXT NOT NULL, rotary_ticks TEXT NOT NULL, slider_border TEXT NOT NULL, slider_hover TEXT NOT NULL, plot_main TEXT NOT NULL, plot_total_curve TEXT NOT NULL, rotary_hover TEXT NOT NULL, slider_indicator TEXT NOT NULL, slider_active TEXT NOT NULL, plot_single_filter TEXT NOT NULL, plot_filter_hover TEXT NOT NULL)",
        (), // empty list of parameters.
    )?;

    conn.execute("insert into themes(name, rotary_ticks, rotary_hover, slider_border, slider_indicator, slider_hover, slider_active, plot_main, plot_single_filter, plot_total_curve, plot_filter_hover) values ('RGB', '#ff0000', '#0000ff', '#0000ff', '#000000', '#00ff00', '#00ff00', '#888888', '#ffffff', '#00ff00', '#ff0000')", ())?;

        Ok(())
}

#[tauri::command]
pub fn sql_create(app_handle: AppHandle) -> Result<(), String> {
    let p = app_handle
        .path_resolver()
        .resource_dir()
        .expect("failed to resolve resource")
        .into_os_string()
        .into_string()
        .unwrap();

    if let Err(e) = create(p) {
        return Err(e.to_string());
    }
    Ok(())
}

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
pub fn sql_query_theme_name(app_handle: AppHandle) -> Result<Theme, String> {
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
pub fn sql_query_theme(theme: Theme, app_handle: AppHandle) -> Result<Colors, String> {
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
    let mut stmt = conn.prepare("SELECT * FROM settings")?;
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

#[tauri::command]
pub fn sql_query_settings(app_handle: AppHandle) -> Result<Settings, String> {
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
