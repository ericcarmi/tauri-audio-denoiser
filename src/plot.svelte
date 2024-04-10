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

  export let settings: any;

  // function pointer instead of if statements inside loop
  // when setting changes, plot_scale points to mel or bark
  let set_plot_scale = (x: number) => {
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
  };
  let max_plot_freq = set_plot_scale(NYQUIST);

  let is_loading = false;

  let webglp: WebglPlot;
  let line: WebglLine;
  let canvasMain: any;
  let freqcanvas: any;

  $: settings, update_colors();

  function update_colors() {
    if (settings) {
      plot_total_curve = rgbToHex(settings.colors.plot_total_curve);
      eq_color = rgbToHex(settings.colors.plot_single_filter);
      eq_hover_color = rgbToHex(settings.colors.plot_filter_hover);
      plot_scale = settings.plot_scale;
      max_plot_freq = set_plot_scale(NYQUIST);
      update_filter_bank(true);
    }
  }

  onMount(() => {
    update_colors();
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

  async function get_time_data() {
    if (selectedRecording === "") return;
    // check cache...maybe
    is_loading = true;
    invoke("get_time_data", { path: selectedRecording }).then((res) => {
      let data: any = res;
      let renderPlot = () => {
        line = new WebglLine(
          new ColorRGBA(140 / 255, 0, 180 / 255, 1),
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

  var last_bar_heights = Array(256).fill(0);

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
          context.fillStyle = "rgb(140,0,180)";
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
      })
      .catch((e) => {
        console.error(e);
      });
  }

  $: bpf_hovering, update_filter_bank(true);

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
          (-sum_curve[i] * FREQ_PLOT_HEIGHT) / 128 + FREQ_PLOT_HEIGHT / 2
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
            (-curve[i] * FREQ_PLOT_HEIGHT) / 128 + FREQ_PLOT_HEIGHT / 2
          );
        }
        context.lineWidth = 2;
        context.strokeStyle = bpf_hovering[idx] ? eq_hover_color : eq_color;
        context.stroke();
      });
    }
    requestAnimationFrame(renderPlot);
  }

  $: selectedRecording, get_time_data();
  $: fft_data, update_fft();
  $: bpf_filters, !is_playing && update_filter_bank(true);
</script>

<div>
  <canvas id="freq_canvas" />
  <canvas id="time_canvas" />
  {#if is_loading}
    <div class="spinner" />
  {/if}
</div>

<style>
  canvas {
    border: 2px solid var(--purple);
    background: black;
  }
  div {
    user-select: none;
  }
  .spinner {
    position: absolute;
    top: calc(60% - 1em);
    left: calc(50% - 1em);
    width: 2em;
    height: 2em;
    border: 3px solid var(--gray150);
    border-bottom-color: var(--purple);
    border-top-color: var(--purple);
    border-radius: 20px;
    animation: 2s infinite linear spin;
  }
</style>
