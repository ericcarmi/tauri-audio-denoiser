<script lang="ts" context="module">
  /**
   * audio params to be used in the audio thread -- some variables can be set directly from messages, others are computed (spectra, sdft) and can skip being serialized
   */
  export type AudioParams = {
    ui_params: UIParams;
    time: number;
    dft_size: number;
  };

  /**
   * user-facing params that control a bandpass filter, convert to IIR for internal audio processing
   */
  export type BPF = { gain: number; freq: number; Q: number };

  export type Complex = { re: number; im: number };

  /**
   * FilterBank -- holds IIR2 filters, might want to store as vec? or some other collection...
   */
  export type FilterBank = {
    bp1: IIR2;
    bp2: IIR2;
    bp3: IIR2;
    bp4: IIR2;
    bp5: IIR2;
  };

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

  export type UIFilterBank = {
    bp1: BPF;
    bp2: BPF;
    bp3: BPF;
    bp4: BPF;
    bp5: BPF;
  };

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
    filter_bank: UIFilterBank;
  };
</script>
