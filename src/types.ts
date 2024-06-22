export type AudioParams = {
  ui_params: UIParams;
  time: number;
  dft_size: number;
};

/**
 * user-facing params that control a bandpass filter, convert to IIR for internal audio processing
 */
export type BPF = { gain: number; freq: number; Q: number };

export type Colors = "RotaryTick" | "RotaryHover";

export type Complex = { re: number; im: number };

export type ComponentColors = {
  rotary_tick: string;
  rotary_hover: string;
  slider_background: string;
  slider_hover: string;
  slider_border: string;
  slider_active: string;
  slider_indicator: string;
  plot_main: string;
  plot_single_filter: string;
  plot_total_curve: string;
  plot_filter_hover: string;
  app_background: string;
  app_text: string;
};

export type Filters = { bank: [IIR2, IIR2, IIR2, IIR2, IIR2] };

/**
 * IIR filter, second order
 */
export type IIR2 = {
  b0: number;
  b1: number;
  b2: number;
  a0: number;
  a1: number;
  a2: number;
  x: [number, number];
  y: [number, number];
};

export type PlotScale = "Linear" | "Mel" | "Log" | "Bark";

export type Settings = {
  id: number;
  plot_scale: PlotScale;
  theme: Theme;
  draw_freq_axis: boolean;
  draw_fft_amp_axis: boolean;
  draw_filter_amp_axis: boolean;
};

export type StereoChoice = "Left" | "Right" | "Both";

/**
 * stereo params includes AudioParams for each channel as well as other params that are independent of the channels
 */
export type StereoParams = {
  left: AudioParams;
  right: AudioParams;
  stereo_choice: StereoChoice;
  clean: boolean;
  num_file_samples: number;
  file_path: string;
  is_stereo: boolean;
  time: number;
};

export type Theme = "RGB" | "CYM" | "POG" | "BWG" | "SEPIA" | "CUSTOM";

export type UIFilters = { bank: [BPF, BPF, BPF, BPF, BPF] };

/**
 * ui params -- states of everything in the ui, does not include everything that can be sent in a Message (file name and a few others), just the stuff that gets stored in db
 */
export type UIParams = {
  clean: boolean;
  left_mute: boolean;
  right_mute: boolean;
  stereo_choice: StereoChoice;
  output_gain: number;
  noise_gain: number;
  pre_smooth_gain: number;
  post_smooth_gain: number;
  filters: UIFilters;
};
