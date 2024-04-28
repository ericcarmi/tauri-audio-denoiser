#!/bin/sh
rm db.sqlite

red=#ff0000
green=#00ff00
blue=#0000ff
cyan=#00ffff
yellow=#ffff00
magenta=#ff00ff
purple=#800080
orange=#cc8800

sqlite3 -batch db.sqlite "CREATE TABLE SETTINGS(id INTEGER PRIMARY KEY, plot_scale TEXT NOT NULL, theme TEXT NOT NULL, draw_freq_axis BOOLEAN, draw_fft_amp_axis BOOLEAN, draw_filter_amp_axis BOOLEAN);
insert into settings (plot_scale, theme, draw_freq_axis, draw_fft_amp_axis, draw_filter_amp_axis) values ('Log', 'RGB', true, true, true);
CREATE TABLE THEMES(id INTEGER PRIMARY KEY, name TEXT NOT NULL, rotary_ticks TEXT NOT NULL, slider_border TEXT NOT NULL, slider_hover TEXT NOT NULL, plot_main TEXT NOT NULL, plot_total_curve TEXT NOT NULL, rotary_hover TEXT NOT NULL, slider_indicator TEXT NOT NULL, slider_active TEXT NOT NULL, plot_single_filter TEXT NOT NULL, plot_filter_hover TEXT NOT NULL);

insert into themes(name, rotary_ticks, rotary_hover, slider_border, slider_indicator, slider_hover, slider_active, plot_main, plot_single_filter, plot_total_curve, plot_filter_hover) values ('RGB', '#ff0000', '#0000ff', '#0000ff', '#000000', '#00ff00', '#00ff00', '#888888', '#ffffff', '#00ff00', '#ff0000' );


insert into themes(name, rotary_ticks, rotary_hover, slider_border, slider_indicator, slider_hover, slider_active, plot_main, plot_single_filter, plot_total_curve, plot_filter_hover) values ('CYM', '$cyan', '$magenta', '$magenta', '#000000', '$yellow', '$yellow', '#888888', '#ffffff', '$yellow', '$cyan');


insert into themes(name, rotary_ticks, rotary_hover, slider_border, slider_indicator, slider_hover, slider_active, plot_main, plot_single_filter, plot_total_curve, plot_filter_hover) values ('POG', '$purple', '$green', '$orange', '#000000', '$orange', '$purple', '#888888', '#ffffff', '$green', '$purple' );


"

cp db.sqlite src-tauri/
cp db.sqlite src-tauri/target/debug