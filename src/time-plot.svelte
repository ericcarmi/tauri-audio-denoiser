<script lang="ts">
	import { invoke } from "@tauri-apps/api/tauri";
	import { onDestroy, onMount } from "svelte";
	import { hexToRgb } from "./functions.svelte";
	import { TIME_PLOT_HEIGHT, TIME_PLOT_WIDTH } from "./constants.svelte";
	import { listen } from "@tauri-apps/api/event";

	type Highlight = {
		left_px: number;
		right_px: number;
		left_time: number;
		right_time: number;
	};

	let indicator_width = 1;
	let indicator_origin = 0;
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
	let origin = 0;

	let is_time_slider_dragging = false;
	let el: HTMLElement;
	let highlight_el: HTMLElement;
	let canvas_el: HTMLCanvasElement;
	let time_ind_el: HTMLElement;
	let indicator_position: number = 0;
	let mouse_down = false;

	let is_loading = false;

	let canvasMain: any;

	let zoom = 1;
	let time_scroll_position = 0;
	let time_scroll_dragging = false;
	// 0.1 is zoom delta, should set that too...
	let delta_translate = TIME_PLOT_WIDTH * 0.1;
	let max_zoom = 5;
	let min_zoom = 1;
	let zoom_delta = 0.1;
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
		time_ind_el.style.left =
			Math.min(
				Math.max(canvas_el.offsetLeft, 0),
				canvas_el.offsetLeft + TIME_PLOT_WIDTH,
			) + "px";

		draggable();
		canvasMain = document.getElementById("time-canvas");
		canvasMain.width = TIME_PLOT_WIDTH;
		canvasMain.height = TIME_PLOT_HEIGHT;
	});

	let redraw_timer = 100;
	let interval: any;

	function resetInterval() {
		clearInterval(interval);
		interval = setInterval(() => {
			redraw_timer += 1;
		}, 1);
	}
	function reset_timer() {
		if (redraw_timer > 10) {
			clearInterval(interval);
			interval = undefined;
			redraw_timer = 0;
			// redraw_time_data();
		}
	}
	// $: redraw_timer, reset_timer();

	function redraw_time_data() {
		let renderPlot = () => {
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
			r = Math.max(1, r);
			// let min_pixel = (highlight_left / num_time_samples) * TIME_PLOT_WIDTH;
			// let min_pixel = (highlight_left / num_time_samples) * TIME_PLOT_WIDTH;
			let min_pixel = highlight_left;
			let max_pixel = highlight_right;

			for (let i = 0; i < TIME_PLOT_WIDTH; i++) {
				if (i >= min_pixel && i <= max_pixel) {
					context.fillStyle = "rgb(0,0,150)";
					context.fillRect(i, 0, i + 1, height);
				} else {
					context.fillStyle = "rgb(0,0,0)";
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
				if (!mouse_down || is_time_slider_dragging) return;

				indicator_origin = origin;
				indicator_width = Math.abs(e.clientX - origin);
				if (e.clientX - origin < 0) {
					// this also needs to take zoom into account...and the translation
					highlight_left = origin - indicator_width - canvas_el.offsetLeft;
					highlight_right = origin - canvas_el.offsetLeft;
					indicator_origin = origin - indicator_width;
				} else if (e.clientX - origin > 0) {
					highlight_left = origin - canvas_el.offsetLeft;
					highlight_right = origin + indicator_width - canvas_el.offsetLeft;
					indicator_position = origin - canvas_el.offsetLeft;
				}
			}
			function reset() {
				mouse_down = false;
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
		// if (!interval) {
		// 	redraw_timer = 0;
		// 	resetInterval();
		// }

		if (e.deltaY > 0) {
			zoom = min(zoom + zoom_delta, max_zoom);
			if (zoom != max_zoom) {
				let r = hover_position / TIME_PLOT_WIDTH / zoom;
				let d = 800 * Math.abs(e.deltaY);
				start = round(Math.min(start + (d * r) / 2, end - TIME_PLOT_WIDTH));
				end = round(Math.max(end - (d * (1 - r)) / 2, start + TIME_PLOT_WIDTH));

				origin = hover_position;
				time_scroll_position = min(
					time_scroll_position + delta_translate * r,
					time_scroll_position + TIME_PLOT_WIDTH / zoom,
				);
				canvas_el.style.transformOrigin = `${origin}px 0`;
				canvas_el.style.transform = `scale(${zoom}, 1) `;
			}
		} else if (e.deltaY < 0) {
			zoom = max(zoom - zoom_delta, min_zoom);
			let r = hover_position / TIME_PLOT_WIDTH / zoom;
			let d = 800 * Math.abs(e.deltaY);
			start = round(Math.max(start - (d * r) / 2, 0));
			end = round(Math.min(end + (d * (1 - r)) / 2, num_time_samples));

			origin = hover_position;
			time_scroll_position = max(time_scroll_position - delta_translate * r, 0);
			canvas_el.style.transformOrigin = `${origin}px 0`;
			canvas_el.style.transform = `scale(${zoom}, 1) `;
		}
		console.log(origin);
	}

	function update_time_ind() {
		if (!is_time_slider_dragging && time_ind_el) {
			time_ind_el.style.left = time_position + canvas_el.offsetLeft + "px";
		}
	}
	$: time_position, update_time_ind();
</script>

<div class="plot-wrapper">
	<input
		style="width: calc({TIME_PLOT_WIDTH}px + 2em);"
		class="time-slider"
		type="range"
		data-attribute={is_time_slider_dragging}
		min={0}
		max={TIME_PLOT_WIDTH}
		bind:value={time_position}
		on:mousedown={(e) => {
			is_time_slider_dragging = true;
			time_ind_el.style.left =
				Math.min(
					Math.max(canvas_el.offsetLeft, e.clientX),
					canvas_el.offsetLeft + TIME_PLOT_WIDTH,
				) + "px";
		}}
		on:mousemove={(e) => {
			if (is_time_slider_dragging) {
				time_ind_el.style.left =
					Math.min(
						Math.max(canvas_el.offsetLeft, e.clientX),
						canvas_el.offsetLeft + TIME_PLOT_WIDTH,
					) -
					0 +
					"px";
			}
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
	<div
		class="time-indicator"
		style="height:{TIME_PLOT_HEIGHT}px"
		bind:this={time_ind_el}
	/>
	<span style="left: 0; position: absolute;"
		>{(start / sampling_rate).toFixed(1)}</span
	>
	<span style="right: 0; position: absolute;"
		>{(end / sampling_rate).toFixed(1)}</span
	>
	<div
		class="scroll-container"
		style="width: {TIME_PLOT_WIDTH}px; "
		role="cell"
		tabindex={0}
		bind:this={el}
	>
		{#if mouse_down}
			<div
				class="highlight"
				bind:this={highlight_el}
				role="marquee"
				style="height: {TIME_PLOT_HEIGHT}px; left: {indicator_origin}px; width: {indicator_width}px"
				data-attribute={Math.abs(indicator_position - indicator_origin) > 0}
				on:mousemove={(e) => {
					// hover_position = e.clientX - canvas_el.offsetLeft;
				}}
			></div>
		{/if}
		<canvas
			id="time-canvas"
			bind:this={canvas_el}
			on:scroll={(e) => {
				e.preventDefault();
			}}
			on:wheel={handleZoom}
			on:mousedown={(e) => {
				if (!is_time_slider_dragging) {
					mouse_down = true;
					indicator_position = e.clientX;
					indicator_origin = e.clientX;
				}
			}}
			on:mousemove={(e) => {
				// need to update this to use zoom
				// it needs to use the bounds too
				hover_position = e.offsetX;

				if (!mouse_down) {
					origin = hover_position + canvas_el.offsetLeft;
				}
			}}
		/>
		<div
			id="time-scrollbar"
			role="button"
			tabindex={0}
			on:mousedown={(e) => {
				// it should work like the other custom draggables but it is not
				// ...should be able to move mouse outside with mouse down and still work
				time_scroll_dragging = true;
				time_scroll_position = e.clientX;
			}}
			on:mouseup={(e) => {
				time_scroll_dragging = false;
			}}
			on:mousemove={(e) => {
				if (time_scroll_dragging) {
					time_scroll_position = e.clientX;
				}
			}}
		>
			<div
				id="time-scroll-position"
				role="marquee"
				style="position: absolute; left: {time_scroll_position +
					canvas_el?.offsetLeft}px; width: calc({TIME_PLOT_WIDTH / zoom}px)"
				on:mousemove={(e) => {
					if (time_scroll_dragging) {
						// time_scroll_position = e.offsetX;
					}
				}}
			/>
		</div>
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
		background: black;
		scale: 1 1;
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
		background: black;
		appearance: none;
		-webkit-appearance: none;
		height: 2em;
		width: 2em;
		clip-path: polygon(0% 0%, 100% 0%, 50% 100%);
		border-radius: 5px;
		position: relative;
		left: 1px;
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
		background: rgba(0, 0, 255, 0.5);
		z-index: 1;
		opacity: 1;
	}
	.scroll-container {
		overflow: hidden;
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
		bottom: -0.5em;
		font-size: 10px;
	}
	.time-indicator {
		width: 1px;
		background: rgb(255, 255, 255, 0.4);
		position: absolute;
		margin-top: 2em;
		z-index: 1;
	}
</style>
