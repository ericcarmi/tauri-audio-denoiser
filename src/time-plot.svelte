<script lang="ts">
	import { invoke } from "@tauri-apps/api/tauri";
	import { onDestroy, onMount } from "svelte";
	import { WebglPlot, WebglLine, ColorRGBA } from "webgl-plot";
	import { hexToRgb } from "./functions.svelte";
	import {
		FREQ_PLOT_WIDTH,
		TIME_PLOT_HEIGHT,
		TIME_PLOT_WIDTH,
	} from "./constants.svelte";
	import { listen } from "@tauri-apps/api/event";

	export let is_playing: boolean;
	export let plot_color: string;
	export let time_data: Array<number> = [];
	export let time_position: number;
	export let time: number;
	export let num_time_samples: number;

	const unlisten_2 = listen("audioui_message", (event: any) => {
		time = event.payload.time / num_time_samples;
		time_position = time * FREQ_PLOT_WIDTH;
	});

	onDestroy(() => {
		unlisten_2.then((f) => f());
	});

	let is_time_slider_dragging = false;

	let is_loading = false;

	let webglp: WebglPlot;
	let line: WebglLine;
	let canvasMain: any;

	$: time_data, redraw_time_data();

	let rgb_color: any = { r: 140, g: 0, b: 180 };
	$: plot_color, (rgb_color = hexToRgb(plot_color));

	onMount(() => {
		canvasMain = document.getElementById("time_canvas");
		canvasMain.width = TIME_PLOT_WIDTH;
		canvasMain.height = TIME_PLOT_HEIGHT;

		webglp = new WebglPlot(canvasMain);
		const numX = 1000;

		line = new WebglLine(new ColorRGBA(1, 0, 0, 1), numX);
		webglp.addLine(line);
		line.arrangeX();
	});

	function redraw_time_data() {
		let renderPlot = () => {
			line = new WebglLine(
				new ColorRGBA(
					rgb_color.r / 255,
					rgb_color.g / 255,
					rgb_color.b / 255,
					1
				),
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
		time_data && requestAnimationFrame(renderPlot);
	}
</script>

<div class="plot-wrapper">
	<div style="width: {FREQ_PLOT_WIDTH}px; ">
		<canvas id="time_canvas" />
		{#if is_loading}
			<div class="spinner" />
		{/if}

		<input
			style="width: {FREQ_PLOT_WIDTH}px;"
			class="time-slider"
			type="range"
			data-attribute={is_time_slider_dragging}
			min={0}
			max={FREQ_PLOT_WIDTH}
			bind:value={time_position}
			on:mousedown={() => {
				is_time_slider_dragging = true;
			}}
			on:mouseup={() => {
				is_time_slider_dragging = false;
			}}
			on:input={() => {
				time = time_position / FREQ_PLOT_WIDTH;

				// only send message when playing, otherwise after dragging the slider, the first message received is the location it started in
				is_playing &&
					invoke("message_time", {
						time: time * num_time_samples,
					});
			}}
		/>
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

	div {
		user-select: none;
	}
	.spinner {
		position: absolute;
		top: calc(44.7% - 1em);
		left: calc(50% - 1em);
	}

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
		background: var(--slider-active);
	}
	input[type="range"][data-attribute="true"]::-webkit-slider-thumb {
		background: var(--slider-active);
	}

	input[type="range"]::-webkit-slider-runnable-track {
		background: var(--gray100);
	}
	.time-slider {
		align-self: center;
		border: 2px solid var(--slider-border);
		transition: border 0.33s;
	}
	.time-slider[data-attribute="true"] {
		border: 2px solid var(--slider-active);
	}
	.time-slider:hover {
		border: 2px solid var(--slider-hover);
	}
</style>
