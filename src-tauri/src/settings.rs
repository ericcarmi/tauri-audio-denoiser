use std::mem::size_of;

use rusqlite::{types::FromSql, ToSql};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

// to make settings iterable, needs to be enum because it holds different types
// pub enum SettingsTypes {
//     Id(i32),
//     PlotScale(PlotScale),
//     Theme(String),
//     DrawFreqAxis(bool),
// }

#[derive(Serialize, Deserialize, Debug, TS)]
#[ts(export)]
pub struct Settings {
    pub id: i32,
    pub plot_scale: PlotScale,
    pub theme: Theme,
    pub draw_freq_axis: bool,
    pub draw_fft_amp_axis: bool,
    pub draw_filter_amp_axis: bool,
    // pub fft_plot_decay: f32,
    // pub fft_plot_size: usize,
    // pub colors: Colors,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            id: 0,
            plot_scale: PlotScale::Linear,
            draw_freq_axis: true,
            draw_fft_amp_axis: true,
            draw_filter_amp_axis: true,
            theme: Theme::POG,
            // fft_plot_decay: 0.8,
            // fft_plot_size: 256,
            // colors: Colors::default(),
        }
    }
}

// impl Settings {
//     pub fn as_array(&self) ->
// }

#[derive(Serialize, Deserialize, Debug, TS)]
#[ts(export)]
pub enum PlotScale {
    Linear,
    Mel,
    Log,
    Bark,
}

impl ToSql for PlotScale {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        use PlotScale::*;
        match self {
            Linear => "Linear".to_sql(),
            Mel => "Mel".to_sql(),
            Log => "Log".to_sql(),
            Bark => "Bark".to_sql(),
        }
    }
}

impl FromSql for PlotScale {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        use PlotScale::*;
        if let Ok(v) = value.as_str() {
            return match v {
                "Linear" => Ok(Linear),
                "Mel" => Ok(Mel),
                "Log" => Ok(Log),
                "Bark" => Ok(Bark),
                _ => Err(rusqlite::types::FromSqlError::InvalidType),
            };
        }
        Err(rusqlite::types::FromSqlError::InvalidType)
    }
}

#[derive(Serialize, Deserialize, Debug, TS)]
#[ts(export)]
pub enum Theme {
    RGB,
    CYM,
    POG,
    BWG,
    SEPIA,
    CUSTOM,
}

impl ToSql for Theme {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        use Theme::*;
        match self {
            RGB => "RGB".to_sql(),
            CYM => "CYM".to_sql(),
            POG => "POG".to_sql(),
            BWG => "BWG".to_sql(),
            SEPIA => "SEPIA".to_sql(),
            CUSTOM => "CUSTOM".to_sql(),
        }
    }
}

impl FromSql for Theme {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        use Theme::*;
        if let Ok(v) = value.as_str() {
            return match v {
                "RGB" => Ok(RGB),
                "CYM" => Ok(CYM),
                "POG" => Ok(POG),
                "BWG" => Ok(BWG),
                "SEPIA" => Ok(SEPIA),
                "CUSTOM" => Ok(CUSTOM),
                _ => Err(rusqlite::types::FromSqlError::InvalidType),
            };
        }
        Err(rusqlite::types::FromSqlError::InvalidType)
    }
}

impl Theme {
    pub fn as_str(&self) -> &str {
        use Theme::*;
        match self {
            RGB => "RGB",
            CYM => "CYM",
            POG => "POG",
            BWG => "BWG",
            SEPIA => "SEPIA",
            CUSTOM => "CUSTOM",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, &str> {
        use Theme::*;
        match s {
            "RGB" => Ok(RGB),
            "CYM" => Ok(CYM),
            "POG" => Ok(POG),
            "BWG" => Ok(BWG),
            "SEPIA" => Ok(SEPIA),
            "CUSTOM" => Ok(CUSTOM),
            _ => Err("invalid theme string"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, TS)]
#[ts(export)]
pub enum Colors {
    RotaryTick,
    RotaryHover,
}
const NUM_COLORS: usize = size_of::<Colors>() / 24;

impl Colors {
    pub fn as_slice() -> Vec<String> {
        let N = size_of::<Colors>() / 24; // 24 : num bytes in String
        let mut v = vec![];
        for _ in 0..N {
            v.push("".to_string());
        }
        v
    }
}
pub type Color = String;

#[derive(Serialize, Deserialize, Debug, TS)]
#[ts(export)]
pub struct ComponentColors {
    pub rotary_tick: Color,
    pub rotary_hover: Color,
    pub slider_background: Color,
    pub slider_hover: Color,
    pub slider_border: Color,
    pub slider_active: Color,
    pub slider_indicator: Color,
    pub plot_main: Color,
    pub plot_single_filter: Color,
    pub plot_total_curve: Color,
    pub plot_filter_hover: Color,
    pub app_background: Color,
    pub app_text: Color,
    pub button_background: Color,
    pub button_text: Color,
}
const NUM_COMPONENT_COLORS: usize = size_of::<ComponentColors>() / 24; // 24 : num bytes in String

impl ComponentColors {
    // pub fn as_slice() -> Vec<Color> {
    // let N = size_of::<ComponentColors>() / 24; // 24 : num bytes in String
    // let mut v = vec![];
    // for _ in 0..N {
    //     v.push("".to_string());
    // }
    // v
    // }
    // pub fn as_slice() -> [Color; NUM_COMPONENT_COLORS] {
    //     [
    //         "".to_string()
    //     ]
    // }
}

// impl Default for ComponentColors {
//     fn default() -> Self {
//         Self {
//             rotary_tick: LIGHTPURPLE,
//             rotary_hover: GREEN,
//             slider_hover: LIGHTPURPLE,
//             slider_border: PURPLE,
//             slider_active: GREEN,
//             slider_indicator: GREEN,
//             plot_main: PURPLE,
//             plot_single_filter: GRAY100,
//             plot_total_curve: GRAY200,
//             plot_filter_hover: PURPLE,
//         }
//     }
// }

fn hex_to_rgb(hex_string: &str) -> Result<(u8, u8, u8), String> {
    if hex_string.len() != 7 && &hex_string[0..1] != "#" {
        return Err("invalid color".to_string());
    }
    let r = hex_string[1..3].parse::<u8>().unwrap();
    let g = hex_string[3..5].parse::<u8>().unwrap();
    let b = hex_string[5..7].parse::<u8>().unwrap();
    Ok((r, g, b))
}
// fn rgb_to_hex(r: u8, g: u8, b: u8) -> String {
//     let mut hex = "".to_string();
//     // Convert the RGB values to a hexadecimal representation
//     for i in 0..3 {
//         let val = (r >> i & 0xff) as u16;
//         if val <= 9 {
//             hex.push('0');
//         }
//         hex.push(format!("{:X}", val).chars().next().unwrap());
//     }
//     // Return the hexadecimal representation
//     hex
// }
