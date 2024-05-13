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

	const max = Math.max;
	const min = Math.min;
	const round = Math.round;

	export let is_playing: boolean;
	export let plot_color: string;
	export let time_data: Array<number> = [];
	export let time_position: number;
	export let time: number;
	export let num_time_samples: number;
	export let hover_position = 0;
	let origin = 0;

	let is_time_slider_dragging = false;
	let el: HTMLElement;
	let ind_el: HTMLElement;
	let canvas_el: HTMLCanvasElement;
	let indicator_width = 1;
	let indicator_origin = 0;
	let indicator_position: number = 0;
	let ind_left = 0;
	let ind_right = 0;
	let mouse_down = false;

	let is_loading = false;

	let webglp: WebglPlot;
	let line: WebglLine;
	let canvasMain: any;

	let zoom = 1;
	let max_zoom = 5;
	let min_zoom = 1;
	let start = 0;
	let end = 1;
	let time_labels = [0, 0, 0, 0, 0];

	let rgb_color: any = { r: 140, g: 0, b: 180 };
	$: time_data, (is_loading = true), redraw_time_data();
	$: time_data, (end = time_data.length);
	$: plot_color, (rgb_color = hexToRgb(plot_color));

	const unlisten_2 = listen("audioui_message", (event: any) => {
		time = event.payload.time / num_time_samples;
		time_position = time * TIME_PLOT_WIDTH;
	});
	const unlisten_resize = listen("tauri://resize", () => {
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

	let redraw_timer = 100;
	let interval: any;

	function resetInterval() {
		clearInterval(interval);
		interval = setInterval(() => {
			redraw_timer += 1;
		}, 10);
	}
	function reset_timer() {
		if (redraw_timer > 10) {
			clearInterval(interval);
			interval = undefined;
			redraw_timer = 0;
			redraw_time_data();
		}
	}
	$: redraw_timer, reset_timer();

	function redraw_time_data() {
		let renderPlot = () => {
			if (time_data.length > 0) {
				is_loading = true;
			}
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

			// not sure this is right, hop about should use canvas width? or num samples? this is the hop for indexing time data...yeah but want to get there to end at width's end, it works
			let r = Math.round((end - start) / TIME_PLOT_WIDTH);
			r = Math.max(1, r);

			for (let i = 0; i < TIME_PLOT_WIDTH; i += 1) {
				line.setY(i, time_data[start + i * r]);
			}
			webglp.update();
			if (time_data.length > 0) {
				is_loading = false;
			}
		};
		time_data && requestAnimationFrame(renderPlot);
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
					ind_left = indicator_origin - indicator_width - canvas_el.offsetLeft
					ind_right = indicator_origin - canvas_el.offsetLeft
				}
				else {
					ind_left = indicator_origin  - canvas_el.offsetLeft
					ind_right = indicator_origin + indicator_width - canvas_el.offsetLeft
					
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

	function handleZoom(e: WheelEvent) {
		e.preventDefault();
		if (!interval) {
			redraw_timer = 0;
			resetInterval();
		}
		if (e.deltaY < 0) {
			let d = 500 * Math.abs(e.deltaY);
			let r = hover_position / TIME_PLOT_WIDTH;
			start = round(Math.min(start + d * r, end - TIME_PLOT_WIDTH));
			end = round(Math.max(end - d * (1 - r), start + TIME_PLOT_WIDTH));
			origin = hover_position;
		} else if (e.deltaY > 0) {
			let d = 500 * e.deltaY;
			let r = hover_position / TIME_PLOT_WIDTH;
			start = round(Math.max(start - d * r, 0));
			end = round(Math.min(end + d * (1 - r), num_time_samples));
			origin = hover_position;
		}
	}
</script>

<div class="plot-wrapper">
	<div
		class="scroll-container"
		style="width: {TIME_PLOT_WIDTH}px; "
		role="cell"
		tabindex={0}
		bind:this={el}
		on:wheel={handleZoom}
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
		>
		<span class="ind-label" style="left: -2em;">{(ind_left*num_time_samples/SAMPLING_RATE/TIME_PLOT_WIDTH).toFixed(1)}</span>
		<span class="ind-label" style="right: -2em;">{(ind_right*num_time_samples/SAMPLING_RATE/TIME_PLOT_WIDTH).toFixed(1)}</span>
		</div>
		<canvas
			id="time_canvas"
			bind:this={canvas_el}
			on:mousemove={(e) => {
				hover_position = e.offsetX;
			}}
		/>
		<div class="time-axis">
			<span class="time-label">{(start / SAMPLING_RATE).toFixed(1)}</span>
			<span class="time-label">{(end / SAMPLING_RATE).toFixed(1)}</span>
		</div>
	</div>

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

<style>
	.plot-wrapper {
		width: 100%;
		display: flex;
		flex-direction: column;
		align-items: center;
	}

	canvas {
		background: black;
		scale: 1 1;
	}

	div {
		user-select: none;
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
		opacity: 0;
	}
	.indicator[data-attribute="true"] {
		background: rgba(255, 255, 0, 0.4);
		z-index: 1;
		opacity: 1;
	}
	.scroll-container {
		overflow-y: hidden;
	}
	.time-axis {
		display: flex;
		height: 25px;
		justify-content: space-between;
	}
	.time-label {
		font-size: 10px;
	}
	.spinner {
		position: absolute;
		margin-top: 20px;
	}
	.ind-label {
		position: absolute;
		bottom: -20px;
		font-size: 10px;
	}
</style>
