<script lang="ts">
	import { invoke } from "@tauri-apps/api/tauri";
	import { onDestroy, onMount } from "svelte";
	import { hexToRgb } from "./functions.svelte";
	import { TIME_PLOT_HEIGHT, TIME_PLOT_WIDTH } from "./constants.svelte";
	import { listen } from "@tauri-apps/api/event";

	let highlight_width = 1;
	let highlight_origin = 0;
	let highlight_left = 0;
	let highlight_right = 0;

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
	export let sampling_rate: number;
	export let loop_start_time: number;
	export let loop_length: number;

	let last_hover_position = 0;

	let is_time_slider_dragging = false;
	let el: HTMLElement;
	let scroll_el: HTMLElement;
	let highlight_el: HTMLElement;
	let canvas_el: HTMLCanvasElement;
	let time_ind_el: HTMLElement;
	let is_highlighting = false;

	let is_loading = false;

	let canvasMain: any;

	let zoom = 1;
	let time_scroll_position = 0;
	let left_origin = 0;
	let click_time_scroll_position = 0;
	let time_scroll_dragging = false;
	// 0.1 is zoom delta, should set that too...
	let max_zoom = 3;
	let min_zoom = 1;
	let zoom_delta = 0.1;
	let start = 0;
	let end = 1;
	let time_labels = [0, 0, 0, 0, 0];

	$: time_data, (is_loading = true), redraw_time_data();
	$: time_data, (end = time_data.length);
	let rgb_color: any;
	$: plot_color, (rgb_color = hexToRgb("#000000"));

	const unlisten_2 = listen("audioui_message", (event: any) => {
		time = event.payload.time / num_time_samples;
		time_position =
			world_to_screen(time * TIME_PLOT_WIDTH) - canvas_el.offsetLeft;
	});
	const unlisten_resize = listen("tauri://resize", () => {
		highlight_width = 0;
	});

	onDestroy(() => {
		unlisten_2.then((f) => f());
		unlisten_resize.then((f) => f());
	});

	onMount(() => {
		// time_ind_el.style.left =
		// 	Math.min(
		// 		Math.max(canvas_el.offsetLeft, 0),
		// 		canvas_el.offsetLeft + TIME_PLOT_WIDTH,
		// 	) + "px";

		draggable();
		scroll_draggable();
		canvasMain = document.getElementById("time-canvas");
		canvasMain.width = TIME_PLOT_WIDTH;
		canvasMain.height = TIME_PLOT_HEIGHT;
		left_origin = canvas_el?.offsetLeft;
	});

	function redraw_time_data() {
		let renderPlot = () => {
			// is_loading = false;
			// return;
			if (time_data.length > 0) {
				is_loading = true;
			}
			const canvas = canvasMain;
			const context: CanvasRenderingContext2D = canvas.getContext("2d");

			const height = canvasMain.height;
			const width = canvasMain.width;
			// context.clearRect(0, 0, width, height);
			context.strokeStyle = `rgb(${rgb_color.r}, ${rgb_color.g}, ${rgb_color.b})`;
			context.beginPath();
			context.moveTo(0, TIME_PLOT_HEIGHT / 2);
			context.lineWidth = 2;

			// the hop size should depend on zoom
			let r = Math.round((num_time_samples - 0) / TIME_PLOT_WIDTH);
			// console.log(num_time_samples, r);

			r = Math.max(1, r);
			let min_pixel = highlight_left;
			let max_pixel = highlight_right;

			for (let i = 0; i < TIME_PLOT_WIDTH; i++) {
				if (i >= min_pixel && i <= max_pixel) {
					context.fillStyle = "#b87d48";
					context.fillRect(i, 0, i + 1, height);
				} else {
					context.fillStyle = "#d4aa7d";
					context.fillRect(i, 0, i + 1, height);
				}
				context.lineTo(i, 50 * time_data[i * r] + TIME_PLOT_HEIGHT / 2);
			}
			context.stroke();

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
				if (!is_highlighting || is_time_slider_dragging) return;

				if (e.clientX < TIME_PLOT_WIDTH + canvas_el.offsetLeft) {
					highlight_width = Math.abs(
						Math.max(screen_to_world(e.clientX), 0) - highlight_origin,
					);
				}
				if (screen_to_world(e.clientX) - highlight_origin > 0) {
					highlight_left = highlight_origin;
					highlight_right = Math.min(
						highlight_origin + highlight_width,
						TIME_PLOT_WIDTH,
					);
					// console.log(highlight_right, highlight_width);

					loop_start_time = round(
						(highlight_left / TIME_PLOT_WIDTH) * num_time_samples,
					);
					loop_length = round(
						((highlight_right - highlight_left) / TIME_PLOT_WIDTH) *
							num_time_samples,
					);
				} else if (screen_to_world(e.clientX) - highlight_origin < 0) {
					highlight_left = Math.max(highlight_origin - highlight_width, 0);
					highlight_right = highlight_origin;

					loop_start_time = round(
						(highlight_left / TIME_PLOT_WIDTH) * num_time_samples,
					);
					loop_length = round(
						((highlight_right - highlight_left) / TIME_PLOT_WIDTH) *
							num_time_samples,
					);
				}
			}
			function reset() {
				is_highlighting = false;
				is_time_slider_dragging = false;
				redraw_time_data();
				window.removeEventListener("mousemove", mouseHandler);
				window.removeEventListener("mouseup", reset);
			}
			window.addEventListener("mousemove", mouseHandler);
			window.addEventListener("mouseup", reset);
		});
	}

	function scroll_draggable() {
		if (scroll_el === null) {
			return;
		}
		scroll_el.addEventListener("mousemove", function (e: MouseEvent) {
			function mouseHandler(e: MouseEvent) {
				if (scroll_el === null) {
					return;
				}
				if (time_scroll_dragging) {
					time_scroll_position = Math.min(
						Math.max(
							0,
							e.clientX - canvas_el.offsetLeft - click_time_scroll_position,
						),
						TIME_PLOT_WIDTH * (1 - 1 / zoom),
					);
					// left_origin = time_scroll_position;
				}
			}
			function reset() {
				is_time_slider_dragging = false;
				redraw_time_data();
				window.removeEventListener("mousemove", mouseHandler);
				window.removeEventListener("mouseup", reset);
			}
			window.addEventListener("mousemove", mouseHandler);
			window.addEventListener("mouseup", reset);
		});
	}

	function handleZoom(e: WheelEvent) {
		e.preventDefault();

		if (e.deltaY > 0) {
			if (zoom != max_zoom) {
				let D = 1 - zoom / (zoom + zoom_delta);
				let dx = (screen_to_world(e.clientX) * D) / zoom;
				time_scroll_position += dx;
				zoom = min(zoom + zoom_delta, max_zoom);
				last_hover_position = screen_to_world(e.clientX);
			}
		} else if (e.deltaY < 0) {
			if (zoom != min_zoom) {
				let D = zoom / (zoom - zoom_delta) - 1;
				let dx = (screen_to_world(e.clientX) * D) / zoom;
				time_scroll_position = Math.min(
					Math.max(0, time_scroll_position - dx),
					TIME_PLOT_WIDTH * (1 - 1 / (zoom - zoom_delta)),
				);
				zoom = max(zoom - zoom_delta, min_zoom);
				last_hover_position = screen_to_world(e.clientX);
			}
		}
	}

	function screen_to_world(x: number) {
		return Math.round(
			(x - canvas_el?.offsetLeft) / zoom + time_scroll_position,
		);
	}
	function world_to_screen(x: number) {
		return Math.round(
			(x - time_scroll_position) * zoom + canvas_el?.offsetLeft,
		);
	}

	// $: time_position, update_time_ind();
</script>

<div class="plot-wrapper">
	<input
		style="width: calc({TIME_PLOT_WIDTH}px + 0em);"
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

			is_playing &&
				invoke("message_time", {
					time: time * num_time_samples,
				});
		}}
	/>
	<!--
	<div
		class="time-indicator"
		style="height:{TIME_PLOT_HEIGHT}px"
		bind:this={time_ind_el}
	/>
	-->

	<div
		class="scroll-container"
		style=""
		role="cell"
		tabindex={0}
		bind:this={el}
	>
		{#if is_highlighting}
			<div
				class="highlight"
				bind:this={highlight_el}
				role="marquee"
				style="height: {TIME_PLOT_HEIGHT}px; left: {world_to_screen(
					highlight_left,
				)}px; width: {highlight_width * zoom}px"
				data-attribute={Math.abs(highlight_right - highlight_origin) > 0 ||
					Math.abs(highlight_left - highlight_origin) > 0}
			></div>
		{/if}
		<canvas
			style="transform: scale({zoom}, 1) translate({-time_scroll_position}px, 0px); transform-origin: 0 0; transition: transform 0.0s;"
			id="time-canvas"
			bind:this={canvas_el}
			on:scroll={(e) => {
				e.preventDefault();
			}}
			on:wheel={handleZoom}
			on:mousedown={(e) => {
				if (!is_time_slider_dragging) {
					is_highlighting = true;
					// indicator_position = e.clientX;
					highlight_origin = screen_to_world(e.clientX);
					highlight_left = highlight_right = highlight_origin;
				}
			}}
			on:mousemove={(e) => {
				hover_position = screen_to_world(e.clientX);

				if (!is_highlighting) {
					highlight_origin = screen_to_world(e.clientX);
				}
			}}
		/>
		<div
			id="time-scrollbar"
			role="button"
			tabindex={0}
			bind:this={scroll_el}
			on:mousedown={(e) => {
				time_scroll_dragging = true;
				click_time_scroll_position = e.offsetX;
			}}
			on:mouseup={(e) => {
				time_scroll_dragging = false;
			}}
		>
			<div
				id="time-scroll-position"
				role="marquee"
				style="position: relative; left: {time_scroll_position}px; width: calc({TIME_PLOT_WIDTH /
					zoom}px)"
			/>
		</div>
	</div>
	<div style="height: 0; width:{TIME_PLOT_WIDTH + canvas_el?.offsetLeft}px">
		<span
			style="font-size: 12px; float: left; left: -1em; position: relative; top: -40px;"
			>{(start / sampling_rate).toFixed(1)}</span
		>
		<span
			style="font-size: 12px; float: right; right: -1em; position: relative; top: -40px;"
			>{(end / sampling_rate).toFixed(1)}</span
		>
	</div>

	{#if is_loading}
		<div class="spinner" />
	{/if}
</div>

<style>
	.plot-wrapper {
		width: 100%;
		display: flex;
		flex-direction: column;
		align-items: center;
	}

	canvas {
		scale: 1 1;
		padding: 0;
		margin: 0;
		border: none;
	}

	#time-scrollbar {
		width: 100%;
		height: 15px;
		background: black;
		margin-bottom: 5px;
	}
	#time-scroll-position {
		height: 15px;
		background: var(--rotary-tick);
	}

	div {
		user-select: none;
	}

	input[type="range"] {
		appearance: none;
	}
	/* indicator for the slider */
	input[type="range"]::-webkit-slider-thumb {
		background: var(--sepia1);
		appearance: none;
		-webkit-appearance: none;
		height: 1em;
		width: 2px;
		position: relative;
		left: 0px;
	}

	input[type="range"]::-webkit-slider-thumb:active {
		background: var(--slider-active);
	}
	input[type="range"][data-attribute="true"]::-webkit-slider-thumb {
		background: var(--slider-active);
	}

	input[type="range"]::-webkit-slider-runnable-track {
		background: var(--sepia2);
	}
	.time-slider {
		align-self: center;
		border: 0.5px solid var(--slider-border);
		transition: border-color 0.33s;
	}
	.time-slider[data-attribute="true"] {
		border-color: var(--slider-active);
	}
	.time-slider:hover {
		border-color: var(--slider-hover);
	}
	.highlight {
		position: absolute;
		z-index: -1;
		opacity: 0;
	}
	.highlight[data-attribute="true"] {
		background: rgba(from var(--sepia0) r g b / 0.5);
		z-index: 1;
		opacity: 1;
	}
	.scroll-container {
		overflow: hidden;
	}
	.spinner {
		position: absolute;
		margin-top: 20px;
	}
</style>
