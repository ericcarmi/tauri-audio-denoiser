<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { onMount } from "svelte";
  import { WebglPlot, WebglLine, ColorRGBA, WebglSquare } from "webgl-plot";
  import { loglin, linlog, linspace, setRectangle } from "./types.svelte";
  import { PLOT_HEIGHT, PLOT_WIDTH } from "./constants.svelte";

  let is_loading = false;

  let webglp: WebglPlot;
  let line: WebglLine;
  export let selectedRecording: string;
  let squ = new WebglSquare(new ColorRGBA(1, 0, 0, 1));

  let freqwebglp: WebglPlot;
  let freqline: WebglLine;

  let canvasMain: any;
  let ctx: any;
  let freqcanvas: any;
  export const freq = 420;
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
    canvasMain = document.getElementById("time_canvas");
    const devicePixelRatio = window.devicePixelRatio || 1;
    canvasMain.width = PLOT_WIDTH;
    canvasMain.height = PLOT_HEIGHT;

    webglp = new WebglPlot(canvasMain);
    const numX = 1000;

    line = new WebglLine(new ColorRGBA(1, 0, 0, 1), numX);
    webglp.addLine(line);
    line.arrangeX();

    freqcanvas = document.getElementById("freq_canvas");
    freqcanvas.width = PLOT_WIDTH;
    freqcanvas.height = PLOT_HEIGHT;

    freqwebglp = new WebglPlot(freqcanvas);
    gl = freqcanvas.getContext("webgl");

    gl.clearColor(0, 0, 0, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.viewport(0, 0, freqcanvas.width, freqcanvas.height);

    vertShader = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vertShader, vertCode);
    gl.compileShader(vertShader);

    fragShader = gl.createShader(gl.FRAGMENT_SHADER);

    gl.shaderSource(fragShader, fragCode);
    gl.compileShader(fragShader);
    shaderProgram = gl.createProgram();
    gl.attachShader(shaderProgram, vertShader);
    gl.attachShader(shaderProgram, fragShader);
    gl.linkProgram(shaderProgram);
    gl.useProgram(shaderProgram);

    positionAttributeLocation = gl.getAttribLocation(
      shaderProgram,
      "a_position"
    );

    // look up uniform locations
    resolutionUniformLocation = gl.getUniformLocation(
      shaderProgram,
      "u_resolution"
    );
    colorUniformLocation = gl.getUniformLocation(shaderProgram, "u_color");

    positionBuffer = gl.createBuffer();

    gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
    gl.viewport(0, 0, gl.canvas.width, gl.canvas.height);
    gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
    indexBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indexBuffer);

    const indices = [0, 1, 2, 2, 1, 3];
    gl.bufferData(
      gl.ELEMENT_ARRAY_BUFFER,
      new Uint16Array(indices),
      gl.STATIC_DRAW
    );

    var size = 2; // 2 components per iteration
    var type = gl.FLOAT; // the data is 32bit floats
    var normalize = false; // don't normalize the data
    var stride = 0; // 0 = move forward size * sizeof(type) each iteration to get the next position
    var offset = 0; // start at the beginning of the buffer
    gl.vertexAttribPointer(
      positionAttributeLocation,
      size,
      type,
      normalize,
      stride,
      offset
    );

    // bind the buffer containing the indices
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indexBuffer);

    // set the resolution
    gl.uniform2f(resolutionUniformLocation, gl.canvas.width, gl.canvas.height);
    gl.enableVertexAttribArray(positionAttributeLocation);
  });

  async function getData() {
    if (selectedRecording === "") return;
    // check cache...maybe
    is_loading = true;
    invoke("get_time_onefft", { path: selectedRecording }).then((res) => {
      let data: any = res;
      let renderPlot = () => {
        line = new WebglLine(new ColorRGBA(1, 0.35, 0, 1), data[0].length);

        webglp.removeAllLines();
        webglp.addLine(line);
        line.arrangeX();
        let hop = Math.round(data[0].length / PLOT_WIDTH / 8);
        for (let i = 0; i < data[0].length; i += hop) {
          line.setY(i, data[0][i] * 1);
        }
        webglp.update();


        line = new WebglLine(new ColorRGBA(1, 0.35, 0, 1), data[0].length);

        // freqwebglp.removeAllLines();
        // freqwebglp.addLine(line);
        // line.arrangeX();
        // hop = Math.round(data[1].length / PLOT_WIDTH / 4);
        // for (let i = 0; i < data[1].length; i += hop) {
        //   line.setY(i, data[1][i] * 1);
        // }
        // freqwebglp.update();


        is_loading = false;

        // let stft = data[1];
        // let fftsize = stft[0].length;

        // // for log spacing, freqhop and fwidth are not constant
        // let minfreq = 1;
        // let maxfreq = 44100 / 2.0;
        // let fr = 44100 / fftsize / 2;
        // let infreqs = linspace(minfreq, maxfreq, fr);
        // let logfreq = 20;
        // let logfreq2 = 25;

        // var primitiveType = gl.TRIANGLES;
        // var offset = 0;
        // var count = 6;
        // var indexType = gl.UNSIGNED_SHORT;
        // // console.log(stft.length, fftsize)

        // // need to handle both conditions, when lengths are shorter than grid and when they are longer
        // var logfreqs=[]
        // for (var t = 0; t < stft.length; t++) {
        //   for (var freq = 0; freq < fftsize; freq++) {
        //     // logfreq = loglin(infreqs[freq], minfreq, maxfreq);
        //     // logfreq2 = loglin(infreqs[freq + 1], minfreq, maxfreq);

        //     // mel scale
        //     logfreq = 2595 * Math.log10(1 + infreqs[freq] / 700);
        //     if(t == 0) {
        //     logfreqs.push(logfreq);
        //     }

        //     logfreq2 = 2595 * Math.log10(1 + infreqs[freq + 1] / 700);
        //     let i = Math.round((t / stft.length) * plotWidth);
        //     let j = Math.round((logfreq / fftsize) * plotHeight/2);
        //     let w = Math.round(((t + 1) / stft.length) * plotWidth);
        //     let h = Math.round((logfreq2 / fftsize) * plotHeight/2);
        //     // divide by 8 is ok but not sure why...seems to depend on fftsize tho...2 makes more sense

        //     setRectangle(gl, i, plotHeight - j, w, h);

        //     let amp = Math.log10(stft[t][freq] + 1e-6);
        //     if (isNaN(amp)) {
        //       amp = 0.0;
        //     }
        //     gl.uniform4f(colorUniformLocation, amp, amp, amp, 1);

        //     gl.drawElements(primitiveType, count, indexType, offset);
        //   }
        // }
        // console.log(logfreqs)
      };
      requestAnimationFrame(renderPlot);
    });
  }

  // $: selectedRecording, getData();
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
    top: calc(40% - 1em);
    left: calc(50% - 1em);
    width: 2em;
    height: 2em;
    border: 3px solid var(--gray150);
    border-bottom-color: var(--purple);
    border-top-color: var(--purple);
    border-radius: 20px;
    animation: 1s infinite linear spin;

  }
</style>
