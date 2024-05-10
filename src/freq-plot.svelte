<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { onMount } from "svelte";
  import { WebglPlot, WebglLine, ColorRGBA } from "webgl-plot";
  import {
    loglin,
    biquad,
    freq_response,
    mel,
    bark_scale,
    hexToRgb,
  } from "./functions.svelte";
  import {
    type BPF,
    type ComponentColors,
    type Settings,
  } from "./types.svelte";
  import {
    FREQ_PLOT_HEIGHT,
    FREQ_PLOT_WIDTH,
    MIN_FREQ,
    NYQUIST,
    TIME_PLOT_HEIGHT,
    TIME_PLOT_WIDTH,
  } from "./constants.svelte";

  export let bpfs: Array<BPF>;
  export let fft_data: any;
  export let is_playing = false;
  export let num_sliders = 0;
  export let bpf_hovering = Array(num_sliders).fill(false);

  let eq_color: string;
  let eq_hover_color: string;
  let plot_total_curve: string;
  let plot_scale: string;
  let plot_color: any = { r: 140, g: 0, b: 180 };

  let draw_fft_amp_axis = true;
  let draw_filter_amp_axis = true;
  let draw_freq_axis = true;

  export let settings: Settings;
  export let theme: ComponentColors;

  let freq_axis_labels = [20, 500, 1000, 2000, 5000, 10000, 20000];
  let filter_amp_axis_labels = [30, 20, 10, 0, -10, -20, -30];
  let fft_amp_axis_labels = [30, 25, 20, 15, 10, 5, 0];

  var last_bar_heights = Array(256).fill(0);

  // function pointer instead of if statements inside loop
  // when setting changes, plot_scale points to mel or bark...but this still uses if statements for now
  function set_plot_scale(x: number) {
    if (plot_scale === "Log") {
      return loglin(x, MIN_FREQ, NYQUIST);
    }
    if (plot_scale === "Mel") {
      return mel(x);
    }
    if (plot_scale === "Bark") {
      return bark_scale(x);
    } else {
      return x;
    }
  }

  // input frequency, get position, for setting labels
  function get_plot_scale(x: number, scale: string) {
    max_plot_freq = set_plot_scale(NYQUIST);
    if (scale === "Log") {
      return (loglin(x, MIN_FREQ, NYQUIST) / max_plot_freq) * FREQ_PLOT_WIDTH;
    }
    if (scale === "Mel") {
      return (mel(x) / max_plot_freq) * FREQ_PLOT_WIDTH;
    }
    if (scale === "Bark") {
      return (bark_scale(x) / max_plot_freq) * FREQ_PLOT_WIDTH;
    } else {
      return (x / max_plot_freq) * FREQ_PLOT_WIDTH;
    }
  }

  let max_plot_freq = set_plot_scale(NYQUIST);

  let is_loading = false;

  let freqcanvas: any;

  $: settings, update_settings();
  $: theme, update_settings();

  function update_settings() {
    if (settings && theme) {
      plot_total_curve = theme.plot_total_curve;
      eq_color = theme.plot_single_filter;
      eq_hover_color = theme.plot_filter_hover;
      plot_color = hexToRgb(theme.plot_main);
      plot_scale = settings.plot_scale;
      max_plot_freq = set_plot_scale(NYQUIST);
      draw_fft_amp_axis = settings.draw_fft_amp_axis;
      draw_filter_amp_axis = settings.draw_filter_amp_axis;
      draw_freq_axis = settings.draw_freq_axis;
      update_filter_bank(true);
      update_axes();
    }
  }

  function update_axes() {
    draw_freq_axis && update_freq_axis();
    draw_filter_amp_axis && update_filter_amp_axis();
    draw_fft_amp_axis && update_fft_amp_axis();
  }

  onMount(() => {
    plot_scale = "Log";
    freq_axis_labels.map((_i) => {
      // console.log(i, get_plot_scale(i, plot_scale));
    });
    update_settings();
    freqcanvas = document.getElementById("freq_canvas");
    freqcanvas.width = FREQ_PLOT_WIDTH;
    freqcanvas.height = FREQ_PLOT_HEIGHT;
  });

  function update_fft() {
    let data = Promise.resolve(fft_data);
    data
      .then((r: any) => {
        let data = r;

        function renderPlot() {
          const canvas = freqcanvas;

          const height = canvas.height;
          const width = canvas.width;

          const context: CanvasRenderingContext2D = canvas.getContext("2d");
          context.clearRect(0, 0, width, height);
          if (plot_color !== undefined) {
            context.fillStyle = `rgb(${plot_color.r},${plot_color.g}, ${plot_color.b})`;
          } else {
            context.fillStyle = "rgb(140,0,180)";
          }
          const length = data.length;

          let barWidth = (width / length) * 1.0;
          barWidth = 1;

          context.beginPath();
          context.moveTo(0, FREQ_PLOT_HEIGHT / 2);
          // for (let i = 0; i < length; i++) {
          // let logfreq = get_plot_scale(freq_axis_labels[i], plot_scale);
          // }
          for (let i = 0; i < data.length; i++) {
            let value = data[i];
            let logfreq =
              (set_plot_scale((i / length) * NYQUIST) * FREQ_PLOT_WIDTH) /
              max_plot_freq;

            let barHeight = (Math.log10(value + 1) * FREQ_PLOT_HEIGHT) / 2;
            // for filling space in between, vary the bar width...kinda looks better as stem plot
            // let barWidth = (l2 - logfreq)/FREQ_PLOT_WIDTH*length/2
            let h = height - barHeight / 4;

            if (h > 0) {
              last_bar_heights[i] += barHeight;
              context.fillRect(
                logfreq,
                height,
                barWidth,
                -last_bar_heights[i] / 6
              );
            }
            last_bar_heights[i] *= 0.85;

            // last_bar_heights[i] += barHeight/8;

            // context.lineTo(logfreq, height-last_bar_heights[i]);
            // context.lineWidth = 5;
            // context.strokeStyle = `rgb(${plot_color.r},${plot_color.g},${plot_color.b})`;
            // last_bar_heights[i] *= 0.85;
          }
          context.stroke();
        }
        data && requestAnimationFrame(renderPlot);

        update_filter_bank(false);
        update_axes();
      })
      .catch((e) => {
        console.error(e);
      });
  }

  function update_freq_axis() {
    function renderPlot() {
      const canvas = freqcanvas;
      const height = canvas.height;
      const context: CanvasRenderingContext2D = canvas.getContext("2d");
      // context.clearRect(0, 0, width, height);
      context.fillStyle = "rgb(140,0,180)";
      const length = freq_axis_labels.length;

      context.setLineDash([2, 2]);
      context.beginPath();
      for (let i = 0; i < length; i++) {
        let logfreq = get_plot_scale(freq_axis_labels[i], plot_scale);
        context.moveTo(logfreq, 2);
        context.lineTo(logfreq, height - 2);
        context.lineWidth = 2;
        context.strokeStyle = "rgb(50,50,50)";
      }
      context.stroke();
      context.setLineDash([]);
    }
    requestAnimationFrame(renderPlot);
  }

  function update_filter_amp_axis() {
    function renderPlot() {
      const canvas = freqcanvas;
      const height = canvas.height;
      const width = canvas.width;
      const context: CanvasRenderingContext2D = canvas.getContext("2d");
      // context.clearRect(0, 0, width, height);
      // context.fillStyle = "rgb(140,0,180)";
      //   context.strokeStyle = "rgb(50,50,50)";
      const length = freq_axis_labels.length;

      context.setLineDash([2, 2]);
      let delta = FREQ_PLOT_HEIGHT / length;
      context.beginPath();
      for (let i = 0; i < length; i++) {
        let amp = (i + 0.5) * delta;
        context.moveTo(0, amp);
        context.lineTo(width, amp);
        context.lineWidth = 1;
      }
      context.stroke();
      context.setLineDash([]);
    }
    requestAnimationFrame(renderPlot);
  }

  function update_fft_amp_axis() {
    function renderPlot() {
      const canvas = freqcanvas;
      const width = canvas.width;
      const context: CanvasRenderingContext2D = canvas.getContext("2d");
      // context.clearRect(0, 0, width, height);
      context.fillStyle = "rgb(140,0,180)";
      const length = fft_amp_axis_labels.length;

      context.setLineDash([4, 4]);
      let delta = FREQ_PLOT_HEIGHT / length;
      for (let i = 0; i < length; i++) {
        context.beginPath();
        let amp = (i + 1) * delta;
        context.moveTo(0, amp);
        context.lineTo(width, amp);
        context.lineWidth = 1;
        context.stroke();
      }
      context.setLineDash([]);
    }
    requestAnimationFrame(renderPlot);
  }

  function update_filter_bank(should_clear: boolean) {
    function renderPlot() {
      const canvas = freqcanvas;

      const height = canvas.height;
      const width = canvas.width;

      const context: CanvasRenderingContext2D = canvas.getContext("2d");
      if (should_clear) context.clearRect(0, 0, width, height);

      let N = FREQ_PLOT_WIDTH;
      let sum_curve: Array<number> = Array(N).fill(0);

      bpfs.map((filt) => {
        let coeffs = biquad(filt.gain, filt.freq, filt.Q);
        const curve = freq_response(coeffs, N);
        for (let i = 0; i < N; i++) {
          sum_curve[i] += curve[i];
        }
      });

      context.beginPath();
      context.moveTo(0, FREQ_PLOT_HEIGHT / 2);
      for (let i = 0; i < N; i++) {
        // let x = (i / N) * (MAX_FREQ - MIN_FREQ) + MIN_FREQ;
        // let x = (i / N) * NYQUIST;
        let logfreq =
          (set_plot_scale((i / N) * NYQUIST) * FREQ_PLOT_WIDTH) / max_plot_freq;
        context.lineTo(
          logfreq,
          (-sum_curve[i] * FREQ_PLOT_HEIGHT) / 64 + FREQ_PLOT_HEIGHT / 2
        );
      }
      context.lineWidth = 2;
      context.strokeStyle = plot_total_curve;
      context.stroke();

      bpfs.map((filt, idx) => {
        let coeffs = biquad(filt.gain, filt.freq, filt.Q);
        const curve = freq_response(coeffs, N);

        context.beginPath();
        context.moveTo(0, FREQ_PLOT_HEIGHT / 2);
        for (let i = 0; i < N; i++) {
          let logfreq =
            (set_plot_scale((i / N) * NYQUIST) * FREQ_PLOT_WIDTH) /
            max_plot_freq;

          sum_curve[i] += curve[i];
          context.lineTo(
            logfreq,
            (-curve[i] * FREQ_PLOT_HEIGHT) / 64 + FREQ_PLOT_HEIGHT / 2
          );
        }
        context.lineWidth = 2;
        context.strokeStyle = bpf_hovering[idx] ? eq_hover_color : eq_color;
        context.stroke();
      });
    }
    requestAnimationFrame(renderPlot);
  }

  $: bpfs,
    !is_playing && update_filter_bank(true),
    !is_playing && update_axes();
  $: bpf_hovering, update_filter_bank(true), update_axes();
  $: fft_data, update_fft();
</script>

<div class="plot-wrapper">
  <div style="width: {FREQ_PLOT_WIDTH}px; ">
    <canvas id="freq_canvas" />
    <div class="freq-label-box" style="width: {FREQ_PLOT_WIDTH}px;">
      {#each freq_axis_labels as label}
        <span
          class="freq-label"
          style="left: {get_plot_scale(label, plot_scale).toFixed(1)}px;"
          >{label.toFixed(0)}</span
        >
      {/each}
    </div>
    <div class="filter-amp-label-box" style="height: {FREQ_PLOT_HEIGHT}px">
      {#each filter_amp_axis_labels as label, i}
        <span
          class="filter-amp-label"
          style="top: calc({((i + 0.5) * FREQ_PLOT_HEIGHT) /
            filter_amp_axis_labels.length}px - 1em);">{label.toFixed(0)}</span
        >
      {/each}
    </div>
    <div class="fft-amp-label-box" style="height: {FREQ_PLOT_HEIGHT}px">
      {#each fft_amp_axis_labels as label, i}
        <span
          class="fft-amp-label"
          style="top: calc({((i + 1) * FREQ_PLOT_HEIGHT) /
            fft_amp_axis_labels.length}px - 1.5em);">{label.toFixed(0)}</span
        >
      {/each}
    </div>
  </div>
</div>

<style>
  .plot-wrapper {
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  canvas {
    border: 2px solid var(--plot-main);
    background: black;
  }

  .freq-label-box {
    display: flex;
    justify-self: center;
    align-items: center;
    display: flex;
    justify-content: flex-start;
    position: relative;
    top: -5px;
    height: 10px;
  }
  .freq-label {
    position: absolute;
    font-size: 10px;
  }
  .filter-amp-label-box {
    display: flex;
    justify-content: flex-start;
    position: absolute;
    top: 0;
    right: 0;
    width: 2%;
  }
  .filter-amp-label {
    position: absolute;
    font-size: 10px;
    left: 0;
  }
  .fft-amp-label-box {
    display: flex;
    justify-content: flex-start;
    position: absolute;
    top: 0;
    left: 0;
    width: 2.5%;
  }
  .fft-amp-label {
    position: absolute;
    font-size: 10px;
    right: 0;
  }
  div {
    user-select: none;
  }
</style>
