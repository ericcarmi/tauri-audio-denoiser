<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { onMount } from "svelte";
  import { WebglPlot, WebglLine, ColorRGBA, WebglSquare } from "webgl-plot";
  import {
    loglin,
    linlog,
    linspace,
    frequencyToXAxis,
    biquad,
    freq_response,
  } from "./functions.svelte";
  import { type BPF } from "./types.svelte";
  import {
    FREQ_PLOT_HEIGHT,
    FREQ_PLOT_WIDTH,
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

  let is_loading = false;

  let webglp: WebglPlot;
  let line: WebglLine;
  let squ = new WebglSquare(new ColorRGBA(1, 0, 0, 1));

  let freqwebglp: WebglPlot;
  let freqline: WebglLine;

  let canvasMain: any;
  let ctx: any;
  let freqcanvas: any;
  export const freq = 520;
  export const amp = 1;

  let gl: WebGL2RenderingContext;
  let vertCode: any;
  let vertShader: any;
  let fragCode: any;
  let fragShader: any;
  let shaderProgram: any;
  let color_buffer: any;
  let vertex_buffer: any;

  let colorUniformLocation: any;
  let indexBuffer: any;

  let coord: any;
  let color: any;

  var positionAttributeLocation: any;
  var resolutionUniformLocation: any;
  let positionBuffer: any;

  fragCode = `precision mediump float;
uniform vec4 u_color;
void main() {
   gl_FragColor = u_color;
}`;

  vertCode = `attribute vec2 a_position;
uniform vec2 u_resolution;
void main() {
   // convert the rectangle from pixels to 0.0 to 1.0
   vec2 zeroToOne = a_position / u_resolution;

   // convert from 0->1 to 0->2
   vec2 zeroToTwo = zeroToOne * 2.0;

   // convert from 0->2 to -1->+1 (clipspace)
   vec2 clipSpace = zeroToTwo - 1.0;

   gl_Position = vec4(clipSpace * vec2(1, -1), 0, 1);
}`;

  onMount(() => {
    var style = getComputedStyle(document.body);
    eq_color = style.getPropertyValue("--gray100");
    eq_hover_color = style.getPropertyValue("--lightpurple");

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

          for (let i = 0; i < data.length; i++) {
            let value = data[i];

            let logfreq = 2595 * Math.log10(1 + (i * 44100) / 2 / length / 700);
            let l2 =
              2595 * Math.log10(1 + ((i + 1) * 44100) / 2 / length / 700);

            let barHeight = (Math.log10(value + 1) * FREQ_PLOT_HEIGHT) / 2;
            // finding the x location px from the frequency
            // let x = (i * FREQ_PLOT_WIDTH) / length;
            let x = ((logfreq / FREQ_PLOT_WIDTH) * length) / 2;
            // for filling space in between, vary the bar width...kinda looks better as stem plot
            // let barWidth = (l2 - logfreq)/FREQ_PLOT_WIDTH*length/2
            let h = height - barHeight / 4;

            if (h > 0) {
              last_bar_heights[i] += barHeight;
              context.fillRect(x, height, barWidth, -last_bar_heights[i] / 4);
              last_bar_heights[i] *= 0.77;
            }
          }
        }
        data && requestAnimationFrame(renderPlot);

        update_filter_bank(false);
      })
      .catch((e) => {
        console.log(e);
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

      let N = 256;
      let sum_curve: Array<number> = Array(N).fill(0);

      bpf_filters.map((filt, idx) => {
        let coeffs = biquad(filt.gain, filt.freq, filt.Q);
        const curve = freq_response(coeffs, N);

        context.beginPath();
        context.moveTo(0, FREQ_PLOT_HEIGHT / 2);
        for (let i = 0; i < N; i++) {
          let logfreq = 2595 * Math.log10(1 + (i * 44100) / 2 / N / 700);
          let x = ((logfreq / FREQ_PLOT_WIDTH) * N) / 2;

          sum_curve[i] += curve[i];
          context.lineTo(
            x,
            (-curve[i] * FREQ_PLOT_HEIGHT) / 64 + FREQ_PLOT_HEIGHT / 2
          );
        }
        context.lineWidth = 2;
        context.strokeStyle = bpf_hovering[idx] ? eq_hover_color : eq_color;
        context.stroke();
      });

      context.beginPath();
      context.moveTo(0, FREQ_PLOT_HEIGHT / 2);
      for (let i = 0; i < N; i++) {
          let logfreq = 2595 * Math.log10(1 + (i * 44100) / 2 / N / 700);
          let x = ((logfreq / FREQ_PLOT_WIDTH) * N) / 2;
        context.lineTo(
          x,
          (-sum_curve[i] * FREQ_PLOT_HEIGHT) / 64 + FREQ_PLOT_HEIGHT / 2
        );
      }
      context.lineWidth = 2;
      context.strokeStyle = "rgb(200,220,240)";
      context.stroke();
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
