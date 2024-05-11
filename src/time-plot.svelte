<script lang="ts">
	import { invoke } from "@tauri-apps/api/tauri";
	import { onDestroy, onMount } from "svelte";
	import { WebglPlot, WebglLine, ColorRGBA } from "webgl-plot";
	import { hexToRgb } from "./functions.svelte";
	import {
		SAMPLING_RATE,
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
	export let hover_position = 0;

	let is_time_slider_dragging = false;
	let el: HTMLElement;
	let ind_el: HTMLElement;
	let canvas_el: HTMLCanvasElement;
	let indicator_width = 1;
	let indicator_origin = 0;
	let indicator_position: number = 0;
	let mouse_down = false;

	let is_loading = false;

	let webglp: WebglPlot;
	let line: WebglLine;
	let canvasMain: any;

	let zoom = 1;
	let start = 0;
	let end = 1;

	$: start, redraw_time_data();
	$: end, redraw_time_data();
	// $: indicator_position, console.log(indicator_position)
	// $: indicator_width, console.log(indicator_width)

	let rgb_color: any = { r: 140, g: 0, b: 180 };
	$: time_data, redraw_time_data();
	$: time_data, (end = time_data.length);
	$: plot_color, (rgb_color = hexToRgb(plot_color));

	const unlisten_2 = listen("audioui_message", (event: any) => {
		time = event.payload.time / num_time_samples;
		time_position = time * TIME_PLOT_WIDTH;
	});
	const unlisten_resize = listen("tauri://resize", (event: any) => {
		indicator_width = 0;
	});

	onDestroy(() => {
		unlisten_2.then((f) => f());
		unlisten_resize.then((f) => f());
	});

	onMount(() => {
		canvasMain = document.getElementById("time_canvas");
		canvasMain.width = TIME_PLOT_WIDTH;
		canvasMain.height = TIME_PLOT_HEIGHT;

		webglp = new WebglPlot(canvasMain);
		const numX = 1000;

		line = new WebglLine(new ColorRGBA(1, 0, 0, 1), numX);
		webglp.addLine(line);
		line.arrangeX();
		draggable();
	});

	function redraw_time_data() {
		is_loading = true;
		let renderPlot = () => {
			line = new WebglLine(
				new ColorRGBA(
					rgb_color.r / 255,
					rgb_color.g / 255,
					rgb_color.b / 255,
					1
				),
				TIME_PLOT_WIDTH
			);

			webglp.removeAllLines();
			webglp.addLine(line);
			line.arrangeX();

			let hop = 1;

			let r = Math.round((end - start) / TIME_PLOT_WIDTH);
			// if (end - start > TIME_PLOT_WIDTH * 2) {
			// 	hop = Math.round(time_data.length / TIME_PLOT_WIDTH / 8);
			// }
			for (let i = 0; i < TIME_PLOT_WIDTH; i += 1) {
				line.setY(i, time_data[i * r]);
			}
			webglp.update();
		};
		time_data && requestAnimationFrame(renderPlot);
		is_loading = false;
	}

	function draggable() {
		if (el === null) {
			return;
		}
		el.addEventListener("mousemove", function (e: MouseEvent) {
			function mouseHandler(e: MouseEvent) {
				if (el === null) {
					return;
				}

				if (!mouse_down || is_time_slider_dragging) return;

				// if (hover_position <= 0) return;
				// if (ind_el.offsetLeft + indicator_width >= TIME_PLOT_WIDTH) return;

				indicator_position = e.clientX;
				if (indicator_position < el.offsetLeft) {
					indicator_position = el.offsetLeft;
					return;
				}
				if (indicator_position > el.offsetLeft + el.offsetWidth) {
					indicator_position = el.offsetLeft;
					return;
				}
				indicator_width = Math.abs(e.clientX - indicator_origin);
				if (e.clientX - indicator_origin < 0) {
					ind_el.style.left = indicator_position.toString() + "px";
				}
			}
			function reset() {
				mouse_down = false;
				window.removeEventListener("mousemove", mouseHandler);
				window.removeEventListener("mouseup", reset);
			}
			window.addEventListener("mousemove", mouseHandler);
			window.addEventListener("mouseup", reset);
		});
	}
</script>

<div class="plot-wrapper">
	<div
		style="width: {TIME_PLOT_WIDTH}px; "
		role="cell"
		tabindex={0}
		bind:this={el}
		on:mousedown={(e) => {
			if (!is_time_slider_dragging) {
				mouse_down = true;
				indicator_position = e.clientX;
				indicator_origin = e.clientX;
			}
		}}
	>
		<div
			class="indicator"
			bind:this={ind_el}
			role="marquee"
			style="height: {TIME_PLOT_HEIGHT}px; left: {indicator_origin}px; width: {indicator_width}px"
			data-attribute={Math.abs(indicator_position - indicator_origin) > 0}
			on:mousemove={(e) => {
				hover_position = e.clientX - canvas_el.offsetLeft;
			}}
		/>
		<canvas
			id="time_canvas"
			bind:this={canvas_el}
			on:mousemove={(e) => {
				hover_position = e.offsetX;
				// console.log(hover_position);
				// hover_position = Math.min(hover_position, TIME_PLOT_WIDTH);
			}}
			on:wheel={(e) => {
				e.preventDefault();
				if (e.deltaY > 0) {
					if ((hover_position / num_time_samples) * SAMPLING_RATE > 0.5) {
						zoom = Math.min(zoom + 0.01, 10);
					} else {
						zoom = Math.max(zoom - 0.01, 1);
						// start += 1
					}
				} else if (e.deltaY < 0) {
					if ((hover_position / num_time_samples) * SAMPLING_RATE > 0.5) {
						zoom = Math.max(zoom - 0.01, 1);
					} else {
						zoom = Math.min(zoom + 0.01, 10);
						// start -= 1
					}
				}
			}}
		/>
		{#if is_loading}
			<div class="spinner" />
		{/if}

		<input
			style="width: {TIME_PLOT_WIDTH}px;"
			class="time-slider"
			type="range"
			data-attribute={is_time_slider_dragging}
			min={0}
			max={TIME_PLOT_WIDTH}
			bind:value={time_position}
			on:mousedown={() => {
				is_time_slider_dragging = true;
			}}
			on:mouseup={() => {
				is_time_slider_dragging = false;
			}}
			on:input={() => {
				time = time_position / TIME_PLOT_WIDTH;
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
		background: black;
	}

	div {
		user-select: none;
	}
	.spinner {
		position: absolute;
		top: calc(44.7% - 1em);
		left: calc(50% - 1em);
		z-index: 2;
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
	.indicator {
		position: absolute;
		z-index: -1;
	}
	.indicator[data-attribute="true"] {
		background: rgba(255, 255, 0, 0.4);
		z-index: 1;
	}
</style>
