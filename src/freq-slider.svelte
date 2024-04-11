<script lang="ts">
	import { onMount } from "svelte";
	import { linlog, linlog2, loglin } from "./functions.svelte";
	import { MAX_FREQ, MIN_FREQ } from "./constants.svelte";
    import { invoke } from "@tauri-apps/api/tauri";

	export let value: number;
	export let index: number;

	let position = 0;
	let el: HTMLElement;
	let indicator: HTMLElement;
	export let update_server = () => {};

	let width = 150;
	let is_dragging = false;

	let control_max_freq = MAX_FREQ;
	let control_min_freq = 100;

	$: value, redraw();
	function update_value() {
		let x =
			((position - 1) / width) * (control_max_freq - control_min_freq) +
			control_min_freq;
		// let logfreq = linlog(x, control_min_freq, control_max_freq)

		value = x;
	}

	function redraw() {
		if (!is_dragging && indicator !== undefined) {
			position =
				((value - control_min_freq) / (control_max_freq - control_min_freq)) *
					width +
				1;
			// console.log(position)

			indicator.style.left = position + "px";
		}
	}

	function draggable() {
		if (el === null) {
			return;
		}
		indicator.addEventListener("mousedown", function (e: MouseEvent) {
			var offsetX = e.clientX - position;
			function mouseMoveHandler(e: any) {
				if (el === null || !is_dragging) {
					return;
				}
				position = e.clientX - offsetX;
				position = Math.max(Math.min(position, width), 1);
				indicator.style.left = position + "px";
				// value = (0.5 - position / width) * range;
				update_value();
			}
			function reset() {
				// have to call this here...maybe want to change how this is handled later
				is_dragging = false;
				window.removeEventListener("mousemove", mouseMoveHandler);
				window.removeEventListener("mouseup", reset);
			}
			window.addEventListener("mousemove", mouseMoveHandler);
			window.addEventListener("mouseup", reset);
		});
		el.addEventListener("mousedown", function (e: MouseEvent) {
			var offsetX = e.clientX - position;

			function mouseMoveHandler(e: any) {
				if (el === null || !is_dragging) {
					return;
				}
				let wrap_position = el.offsetLeft;
				// change this and other stuff to not use constants that aren't based on the specified dimensions of the component
				if (Math.abs(e.clientX - wrap_position) > 5) {
					position = e.clientX - wrap_position - 5;
					position = Math.max(Math.min(position, width), 1);
				} else {
					position = e.clientX - offsetX;
					position = Math.max(Math.min(position, width), 1);
				}
				indicator.style.left = position + "px";
				// value = (0.5 - position / width) * range;
				update_value();
			}
			function reset() {
				// have to call this here...maybe want to change how this is handled later
				is_dragging = false;

				invoke("save_bpf_freq", { freq: value, index: index });
				window.removeEventListener("mousemove", mouseMoveHandler);
				window.removeEventListener("mouseup", reset);
			}
			window.addEventListener("mousemove", mouseMoveHandler);
			window.addEventListener("mouseup", reset);
		});
	}
	onMount(() => {
		draggable();
	});
</script>

<div
	class="wrapper"
	bind:this={el}
	data-attribute={is_dragging}
	role="button"
	tabindex={-1}
	on:mousedown={(e) => {
		is_dragging = true;
		let wrap_position = el.offsetLeft;
		if (Math.abs(e.clientX - wrap_position) > 5) {
			position = e.clientX - wrap_position - 5;
			position = Math.max(Math.min(position, width), 1);
		}
		indicator.style.left = position + "px";
		// value = (0.5 - position / width) * maxfreq;
		update_value();
	}}
	on:mouseup={() => {
		is_dragging = false;
	}}
>
	<div
		class="thumb"
		bind:this={indicator}
		data-attribute={is_dragging}
		role="button"
		tabindex={-1}
		on:mousedown={() => {
			is_dragging = true;
		}}
		on:mouseup={() => {
			is_dragging = false;
		}}
	/>
	<span class="value-text">{value.toFixed(1)}</span>
</div>

<style>
	.value-text {
		position: relative;
		top: -20%;
		font-size: 12px;
		color: var(--gray200);
		pointer-events: none;
		font-weight: bold;
	}
	.wrapper {
		background: var(--gray100);
		width: 10em;
		height: 1em;
		display: flex;
		justify-content: center;
		border: 1px solid var(--slider-border);
		position: relative;
		transition: border 0.33s;
	}

	.wrapper:hover {
		border: 1px solid var(--slider-hover);
	}

	.wrapper[data-attribute="true"] {
		border: 1px solid var(--slider-hover);
	}

	.thumb {
		background: black;
		width: 0.5em;
		height: 1em;
		position: absolute;
	}

	.thumb:active {
		background: var(--slider-active);
	}

	.thumb[data-attribute="true"] {
		background: var(--slider-active);
	}
</style>
