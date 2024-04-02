<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { onMount } from "svelte";
  import Plot from "./plot.svelte";
  import { FREQ_PLOT_WIDTH, num_sliders } from "./constants.svelte";
  import BandpassSlider from "./bandpass-slider.svelte";
  import type { BPF, FilterBank, FilterCoeffs2 } from "./types.svelte";
  import { biquad } from "./functions.svelte";

  var gains = [10, 0, 0, 0, 0];
  var freqs = [2000, 500, 1000, 200, 10000];
  var Qs = [0.5, 0.5, 0.5, 0.5, 50.5];

  let bpf_filters: Array<BPF> = Array(num_sliders)
    .fill(0)
    .map((_, i) => {
      return { gain: gains[i], freq: freqs[i], Q: Qs[i] };
    });

  let filter_bank: FilterBank;

  // listen("tauri://file-drop", async (event) => {
  // console.log(event.payload);
  // need to handle this getting history of all files dropped...
  // let s = event.payload[0] as string;
  // await invoke("play_wav", { path: s });
  // });

  let time = 0;
  let selectedRecording = "";
  let is_playing = false;

  let clean = false;

  onMount(async () => {
    // const resourcePath = await resolveResource("assets/test-file.wav");
    // const langDe = JSON.parse(await readTextFile(resourcePath));
    // console.log(langDe);
    selectedRecording = "test-file.wav";
    resetInterval();

    let fbank: any = {};
    for (let i = 0; i < num_sliders; i++) {
      let gain = 10;
      let freq = 1000;
      let Q = 1;
      let bpf: BPF = { gain: gain, freq: freq, Q: Q };
      let coeffs = biquad(gain, freq, Q);
      coeffs.x = [0, 0];
      coeffs.y = [0, 0];
      fbank[`bp${i + 1}`] = coeffs;
    }

    filter_bank = fbank as FilterBank;

    // load from server
    let bpfs: Array<BPF> = await invoke("get_global_state");
    bpf_filters = [...bpfs]
  });

  let slider_values = Array(num_sliders).fill(0);

  let perf_time = performance.now();
  let time_origin = 0;
  let time_delta = 0;

  let is_time_slider_dragging = false;

  let interval: any;
  let fft_data: any;

  function resetInterval() {
    clearInterval(interval);
    interval = setInterval(
      () => {
        if (is_playing) {
          perf_time = performance.now();
          time_delta = perf_time - time_origin;
          time += time_delta / 1000;
          fft_data = invoke("get_fft_plot_data");
          // console.log(fft_data)
        }
      },
      // this works for now, just have to call resetInterval after pressing button
      is_playing ? 10 : 1000
    );
  }
</script>

<main class="container">
  <Plot bind:is_playing bind:bpf_filters {selectedRecording} {fft_data} />
  <input
    style="width: {FREQ_PLOT_WIDTH}px;"
    class="time-slider"
    type="range"
    data-attribute={is_time_slider_dragging}
    min={0}
    max={100000}
    bind:value={time}
    on:mousedown={() => {
      is_time_slider_dragging = true;
    }}
    on:mouseup={() => {
      is_time_slider_dragging = false;
    }}
    on:input={async () => {
      await invoke("update_time", { t: time / 100000 });
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

    <button
      on:click={() => {
        let r = invoke("save_global_state", { bpfs: bpf_filters });
        console.log(r);
      }}
    >
      set
    </button>
    <button
      on:click={async () => {
        let r = await invoke("get_global_state");
        console.log(r);
      }}
    >
      get
    </button>
  </div>

  <div
    class="filter-grid"
    style="grid-template-columns:repeat({num_sliders}, auto)"
  >
    {#each slider_values as val, i}
      <BandpassSlider
        bind:bpf={bpf_filters[i]}
        bind:gain={bpf_filters[i].gain}
        bind:freq={bpf_filters[i].freq}
        bind:Q={bpf_filters[i].Q}
        index={i + 1}
      />
    {/each}
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
    background: var(--orange);
  }

  input[type="range"]::-webkit-slider-runnable-track {
    background: var(--gray100);
  }

  .filter-grid {
    display: grid;
    justify-items: center;
    grid-template-rows: auto;
    appearance: none;
    height: 100px;
    border: 2px solid var(--purple);
  }

  .time-slider {
    align-self: center;
    border: 2px solid var(--purple);
    transition: border 0.33s;
  }
  .time-slider[data-attribute="true"] {
    border: 2px solid var(--orange);
  }
  .time-slider:hover {
    border: 2px solid var(--orange);
  }
</style>
