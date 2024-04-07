<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { onMount } from "svelte";
  import Plot from "./plot.svelte";
  import {
    DOWN_RATE,
    FREQ_PLOT_WIDTH,
    SAMPLING_RATE,
    num_sliders,
  } from "./constants.svelte";
  import BandpassSlider from "./bandpass-slider.svelte";
  import type { BPF } from "./types.svelte";
  import { biquad } from "./functions.svelte";
  import RotarySlider from "./rotary-slider.svelte";

  var gains = [10, 0, 0, 0, 0];
  var freqs = [2000, 500, 1000, 200, 10000];
  var Qs = [0.5, 0.5, 0.5, 0.5, 50.5];

  let bpf_filters: Array<BPF> = Array(num_sliders)
    .fill(0)
    .map((_, i) => {
      return { gain: gains[i], freq: freqs[i], Q: Qs[i] };
    });
  let bpf_hovering = Array(num_sliders).fill(false);

  // listen("tauri://file-drop", async (event) => {
  // need to handle this getting history of all files dropped...
  // let s = event.payload[0] as string;
  // });

  let time = 0;
  let time_position = 0;
  let selectedRecording = "";
  let is_playing = false;

  let clean = false;
  let perf_time = performance.now();
  let time_origin = 0;
  let time_delta = 0;
  let num_time_samples = 1;
  let is_time_slider_dragging = false;

  let interval: any;
  let fft_data: any;

  // these values should also be synced with backend on startup, should not require changing sliders...these values are initialized in audio stream setup, might need to call from server from both places?
  let output_gain = 1.0;
  let noise_gain = 0.0;
  let pre_smooth_gain = 0.5;
  let post_smooth_gain = 0.5;
  let noise_variance = 0.0;

  async function get_time_data() {
    if (selectedRecording === "") return;
    return await invoke("get_time_data", { path: selectedRecording }).then(
      (res) => {
        let data: any = res;
        // need to sync this and the downsample rate from the backend
        // or not...when it reads the correct number of samples
        num_time_samples = data.length;
      }
    );
  }

  let time_slider_max = num_time_samples;
  $: num_time_samples, (time_slider_max = num_time_samples);

  onMount(async () => {
    // const resourcePath = await resolveResource("assets/test-file.wav");
    // const langDe = JSON.parse(await readTextFile(resourcePath));
    // console.log(langDe);
    // selectedRecording = "440-7040-whitenoise.wav";
    selectedRecording = "reisman.wav";
    // selected recording also needs to be in sync with backend file...should be resolved once files are imported correctly instead of one by default, tho should still have that for loading saved state?
    get_time_data();
    resetInterval();

    let fbank: any = {};
    for (let i = 0; i < num_sliders; i++) {
      let gain = 10;
      let freq = 1000;
      let Q = 1;
      // let bpf: BPF = { gain: gain, freq: freq, Q: Q };
      let coeffs = biquad(gain, freq, Q);
      coeffs.x = [0, 0];
      coeffs.y = [0, 0];
      fbank[`bp${i + 1}`] = coeffs;
    }

    // filter_bank = fbank as FilterBank;

    // load from server
    let bpfs: Array<BPF> = await invoke("get_global_state");
    bpf_filters = [...bpfs];
  });

  function resetInterval() {
    clearInterval(interval);
    interval = setInterval(
      () => {
        if (is_playing) {
          perf_time = performance.now();
          time_delta = perf_time - time_origin;

          time += 10 / 1000;
          time_position = (time / DOWN_RATE) * SAMPLING_RATE;
          // console.log(time, time_position, perf_time);

          fft_data = invoke("get_fft_plot_data");
          // console.log(fft_data)
        }
      },
      // this works for now, just have to call resetInterval after pressing button
      is_playing ? 10 : 1000
    );
  }

  // $: time, time_position = time*SAMPLING_RATE/DOWN_RATE/time_slider_max, console.log(time_position, time, time_slider_max);

  // for now just using this for the reset all gain switch, so the filters on backend are updated, and server at same time, which usually doesn't happen when moving gain slider (only sends to server on mouse up)
  function update_filters(
    index: number,
    gain: number,
    freq: number,
    Q: number,
    update_server: boolean
  ) {
    let b = biquad(gain, freq, Q);
    b.x = [0, 0];
    b.y = [0, 0];
    if (index == 1) {
      invoke("update_filters", { bp1: b });
      if (update_server) {
        invoke("save_bpf_gain", { gain: gain, index: index });
        invoke("save_bpf_freq", { freq: freq, index: index });
        invoke("save_bpf_Q", { q: Q, index: index });
      }
    } else if (index == 2) {
      if (update_server) {
        invoke("update_filters", { bp2: b });
        invoke("save_bpf_gain", { gain: gain, index: index });
        invoke("save_bpf_freq", { freq: freq, index: index });
        invoke("save_bpf_Q", { q: Q, index: index });
      }
    } else if (index == 3) {
      if (update_server) {
        invoke("update_filters", { bp3: b });
        invoke("save_bpf_gain", { gain: gain, index: index });
        invoke("save_bpf_freq", { freq: freq, index: index });
        invoke("save_bpf_Q", { q: Q, index: index });
      }
    } else if (index == 4) {
      if (update_server) {
        invoke("update_filters", { bp4: b });
        invoke("save_bpf_gain", { gain: gain, index: index });
        invoke("save_bpf_freq", { freq: freq, index: index });
        invoke("save_bpf_Q", { q: Q, index: index });
      }
    } else if (index == 5) {
      if (update_server) {
        invoke("update_filters", { bp5: b });
        invoke("save_bpf_gain", { gain: gain, index: index });
        invoke("save_bpf_freq", { freq: freq, index: index });
        invoke("save_bpf_Q", { q: Q, index: index });
      }
    }
  }
</script>

<main class="container">
  <Plot
    bind:bpf_hovering
    bind:is_playing
    bind:bpf_filters
    {selectedRecording}
    {fft_data}
  />
  <input
    style="width: {FREQ_PLOT_WIDTH}px;"
    class="time-slider"
    type="range"
    data-attribute={is_time_slider_dragging}
    min={0}
    step={1}
    max={time_slider_max}
    bind:value={time_position}
    on:mousedown={() => {
      is_time_slider_dragging = true;
    }}
    on:mouseup={() => {
      is_time_slider_dragging = false;
    }}
    on:input={async () => {
      // time_position = (time / DOWN_RATE) * SAMPLING_RATE;
      // time_origin = time_position*DOWN_RATE/SAMPLING_RATE
      time = (time_position * DOWN_RATE) / SAMPLING_RATE;
      await invoke("update_time", {
        t: (time * SAMPLING_RATE) / num_time_samples / DOWN_RATE,
      });
      // console.log(time, time * SAMPLING_RATE / num_time_samples/DOWN_RATE);
    }}
  />
  <div>
    <button
      on:click={async () => {
        if (!is_playing) {
          await invoke("play_stream");
          is_playing = true;
          time_origin = performance.now();
        } else {
          await invoke("pause_stream");
          // time_origin = time
          is_playing = false;
        }
        resetInterval();
      }}
    >
      {is_playing ? "pause" : "play"}
    </button>
    <button
      on:click={async () => {
        clean = !clean;
        invoke("update_clean", { clean: clean });
      }}
    >
      {clean ? "clean" : "dirty"}
    </button>
  </div>

  <div
    class="filter-grid"
    style="grid-template-columns:repeat({num_sliders}, auto)"
  >
    {#each bpf_filters as _, i}
      <div
        class="bpf-wrap"
        role="button"
        tabindex="0"
        on:mouseenter={() => {
          bpf_hovering[i] = true;
        }}
        on:mouseleave={() => {
          bpf_hovering[i] = false;
        }}
      >
        <BandpassSlider
          bind:gain={bpf_filters[i].gain}
          bind:freq={bpf_filters[i].freq}
          bind:Q={bpf_filters[i].Q}
          index={i + 1}
        />
      </div>
    {/each}
  </div>
  <div
    style="display: flex; flex-direction: row; justify-content: space-evenly;height:auto;"
  >
    <button
      class="reset-all-gains-switch"
      title="reset all gains to 0 dB"
      on:click={() => {
        bpf_filters = bpf_filters.map((filt, i) => {
          update_filters(i + 1, 0.0, filt.freq, filt.Q, true);
          return { gain: 0.0, freq: filt.freq, Q: filt.Q };
        });
      }}
    >reset gains</button>
    <RotarySlider
      bind:value={output_gain}
      index={-1}
      label="out"
      update_backend={() => {
        invoke("update_output_gain", { gain: output_gain });
      }}
      update_server={() => {
        invoke("save_output_gain", { gain: output_gain });
      }}
    />
    <RotarySlider
      bind:value={noise_gain}
      index={-1}
      label="noise"
      max_val={20}
      min_val={-120}
      update_backend={() => {
        invoke("update_noise_gain", { gain: noise_gain });
      }}
      update_server={() => {
        invoke("save_noise_gain", { gain: noise_gain });
      }}
    />
    <RotarySlider
      bind:value={pre_smooth_gain}
      index={-1}
      label="pre_smooth"
      max_val={0.9999}
      min_val={0}
      update_backend={() => {
        invoke("update_pre_smooth_gain", { gain: pre_smooth_gain });
      }}
      update_server={() => {
        // invoke("save_pre_smooth_gain", { gain: pre_smooth_gain });
      }}
    />
    <RotarySlider
      bind:value={post_smooth_gain}
      index={-1}
      label="post_smooth"
      max_val={0.9999}
      min_val={0.7}
      update_backend={() => {
        invoke("update_post_smooth_gain", { gain: post_smooth_gain });
      }}
      update_server={() => {
        // invoke("save_post_smooth_gain", { gain: post_smooth_gain });
      }}
    />
    <RotarySlider
      bind:value={noise_variance}
      index={-1}
      label="noise variance"
      max_val={10}
      min_val={0}
      update_backend={() => {
        invoke("update_noise_variance", { gain: noise_variance });
      }}
      update_server={() => {
        // invoke("save_noise_variance", { gain: noise_variance });
      }}
    />
  </div>
</main>

<style>
  input[type="range"] {
    appearance: none;
  }
  input[type="range"]::-webkit-slider-thumb {
    background: black;
    appearance: none;
    -webkit-appearance: none;
    height: 2em;
    width: 1em;
  }

  input[type="range"]::-webkit-slider-thumb:active {
    background: var(--lightpurple);
  }
  input[type="range"][data-attribute="true"]::-webkit-slider-thumb {
    background: var(--purple);
  }

  input[type="range"]::-webkit-slider-runnable-track {
    background: var(--gray100);
  }

  .filter-grid {
    display: grid;
    justify-items: center;
    grid-template-rows: auto;
    appearance: none;
    height: 100%;
    border: 0px solid var(--purple);
  }

  .time-slider {
    align-self: center;
    border: 2px solid var(--purple);
    transition: border 0.33s;
  }
  .time-slider[data-attribute="true"] {
    border: 2px solid var(--lightpurple);
  }
  .time-slider:hover {
    border: 2px solid var(--lightpurple);
  }
  .bpf-wrap {
    display: flex;
    border: 0px solid var(--purple);
  }

  .reset-all-gains-switch {
    font-size: 14px;
    align-items: center;
    height: 2em;
    border-radius: 0;
    padding: 0;
    margin: 0;
    align-self: flex-end;
    width: max-content;
    border: 1px solid var(--orange);
  }
  .reset-all-gains-switch:hover {
    border-color: var(--lightorange);
  }
</style>
