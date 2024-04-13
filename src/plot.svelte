<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { onMount } from "svelte";
  import { WebglPlot, WebglLine, ColorRGBA, WebglSquare } from "webgl-plot";
  import {
    loglin,
    linlog,
    biquad,
    freq_response,
    linlog2,
    loglin2,
    mel,
    bark_scale,
    rgbToHex,
  } from "./functions.svelte";
  import { type BPF } from "./types.svelte";
  import {
    FREQ_PLOT_HEIGHT,
    FREQ_PLOT_WIDTH,
    MAX_FREQ,
    MIN_FREQ,
    NYQUIST,
    TIME_PLOT_HEIGHT,
    TIME_PLOT_WIDTH,
    num_sliders,
  } from "./constants.svelte";

  export let bpf_filters: Array<BPF>;
  export let fft_data: any;
  export let is_playing = false;
  export let selectedRecording: string;
  export let bpf_hovering = Array(num_sliders).fill(false);

  let eq_color: string;
  let eq_hover_color: string;
  let plot_total_curve: string;
  let plot_scale: string;
  let plot_color: any;

  let draw_fft_amp_axis = true;
  let draw_filter_amp_axis = true;
  let draw_freq_axis = true;

  export let settings: any;

  let time_data: any;

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

  let webglp: WebglPlot;
  let line: WebglLine;
  let canvasMain: any;
  let freqcanvas: any;

  $: settings, update_settings();

  function update_settings() {
    if (settings) {
      plot_total_curve = rgbToHex(settings.colors.plot_total_curve);
      eq_color = rgbToHex(settings.colors.plot_single_filter);
      eq_hover_color = rgbToHex(settings.colors.plot_filter_hover);
      plot_color = settings.colors.plot_main;
      plot_scale = settings.plot_scale;
      max_plot_freq = set_plot_scale(NYQUIST);
      draw_fft_amp_axis = settings.draw_fft_amp_axis;
      draw_filter_amp_axis = settings.draw_filter_amp_axis;
      draw_freq_axis = settings.draw_freq_axis;
      update_filter_bank(true);
      update_axes();
      get_time_data(selectedRecording);
    }
  }

  function update_axes() {
    draw_freq_axis && update_freq_axis();
    draw_filter_amp_axis && update_filter_amp_axis();
    draw_fft_amp_axis && update_fft_amp_axis();
  }

  onMount(() => {
    update_settings();
    canvasMain = document.getElementById("time_canvas");
    canvasMain.width = TIME_PLOT_WIDTH;
    canvasMain.height = TIME_PLOT_HEIGHT;

    webglp = new WebglPlot(canvasMain);
    const numX = 1000;

    line = new WebglLine(new ColorRGBA(1, 0, 0, 1), numX);
    webglp.addLine(line);
    line.arrangeX();

    freqcanvas = document.getElementById("freq_canvas");
    freqcanvas.width = FREQ_PLOT_WIDTH;
    freqcanvas.height = FREQ_PLOT_HEIGHT;
  });

  function redraw_time_data() {
    let renderPlot = () => {
      line = new WebglLine(
        new ColorRGBA(plot_color.r / 255, plot_color.g, plot_color.b / 255, 1),
        time_data.length
      );

      webglp.removeAllLines();
      webglp.addLine(line);
      line.arrangeX();
      let hop = Math.round(time_data.length / TIME_PLOT_WIDTH / 8);
      for (let i = 0; i < time_data.length; i += hop) {
        line.setY(i, time_data[i]);
      }
      webglp.update();

      is_loading = false;
    };
    requestAnimationFrame(renderPlot);
  }

  async function get_time_data(file_path: string) {
    console.log(file_path);

    if (file_path === "" || plot_color === undefined) return;
    // check cache...maybe
    // if (time_data !== undefined) {
    //   redraw_time_data();
    //   return;
    // }
    is_loading = true;
    invoke("get_time_data", { path: file_path }).then((res) => {
    console.log(res)

      let data: any = res;
      time_data = data;
      let renderPlot = () => {
        line = new WebglLine(
          new ColorRGBA(
            plot_color.r / 255,
            plot_color.g,
            plot_color.b / 255,
            1
          ),
          data.length
        );

        webglp.removeAllLines();
        webglp.addLine(line);
        line.arrangeX();
        let hop = Math.round(data.length / TIME_PLOT_WIDTH / 8);
        for (let i = 0; i < data.length; i += hop) {
          line.setY(i, data[i]);
        }
        webglp.update();

        is_loading = false;
      };
      requestAnimationFrame(renderPlot);
    });
  }

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
              last_bar_heights[i] = barHeight;
              context.fillRect(
                logfreq,
                height,
                barWidth,
                -last_bar_heights[i] / 1
              );
            }
            // last_bar_heights[i] *= 0.86;
          }
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
      for (let i = 0; i < length; i++) {
        context.beginPath();
        let logfreq = get_plot_scale(freq_axis_labels[i], plot_scale);
        context.moveTo(logfreq, 2);
        context.lineTo(logfreq, height - 2);
        context.lineWidth = 2;
        context.strokeStyle = "rgb(50,50,50)";
        context.stroke();
      }
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
      for (let i = 0; i < length; i++) {
        context.beginPath();
        let amp = (i + 0.5) * delta;
        context.moveTo(0, amp);
        context.lineTo(width, amp);
        context.lineWidth = 1;
        context.stroke();
      }
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

      bpf_filters.map((filt) => {
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

      bpf_filters.map((filt, idx) => {
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

  $: bpf_filters, !is_playing && update_filter_bank(true), update_axes();
  $: bpf_hovering, update_filter_bank(true), update_axes();
  $: selectedRecording, get_time_data(selectedRecording);
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
    <canvas id="time_canvas" />
    {#if is_loading}
      <div class="spinner" />
    {/if}
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
  .spinner {
    position: absolute;
    top: calc(44.7% - 1em);
    left: calc(50% - 1em);
    width: 2em;
    height: 2em;
    border: 3px solid var(--gray150);
    border-top-color: var(--plot-main);
    border-radius: 20px;
    animation: 1.47s infinite linear spin;
  }
</style>
