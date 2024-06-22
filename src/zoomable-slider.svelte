<script lang="ts">
	import { onMount } from "svelte";
	import { NYQUIST } from "./constants.svelte";

	export let control_max_freq = NYQUIST;
	export let control_min_freq = 20;
	export let value: number = control_max_freq / 2;
	export let index: number;
	export let update_backend = () => {};
	export let update_database = () => {};

	let height = "1.5em";

	let position = 0;
	let el: HTMLElement;
	let indicator: HTMLElement;
	let is_dragging = false;
	let width = 180;
	let indicator_width = 5;

	let delta = 0;
	$: control_max_freq, (delta = (control_max_freq - control_min_freq) / width);
	$: control_min_freq, (delta = (control_max_freq - control_min_freq) / width);

	$: value, el && indicator && update_position();

	function update_value() {
		let x =
			(position / width) * (control_max_freq - control_min_freq) +
			control_min_freq;
		value = x;
		// audio backend must be updated on every change, server only on reset/mouseup
		update_backend();
	}

	function update_position() {
		position =
			((value - control_min_freq) / (control_max_freq - control_min_freq)) *
			width;
		position = Math.max(Math.min(position, width - indicator_width), 0);
		indicator.style.left = position + "px";
	}

	function draggable() {
		if (el === null) {
			return;
		}
		indicator.addEventListener("mousedown", function (e: MouseEvent) {
			var offsetX = e.clientX - position;
			function mouseMoveHandler(e: MouseEvent) {
				if (el === null || !is_dragging) {
					return;
				}

				position = e.clientX - offsetX;
				position = Math.max(Math.min(position, width - indicator_width), 0);
				indicator.style.left = position + "px";
				update_value();
			}
			function reset() {
				is_dragging = false;
				update_database();
				window.removeEventListener("mousemove", mouseMoveHandler);
				window.removeEventListener("mouseup", reset);
			}
			window.addEventListener("mousemove", mouseMoveHandler);
			window.addEventListener("mouseup", reset);
		});
		el.addEventListener("mousedown", function (e: MouseEvent) {
			var offsetX = e.clientX - position;

			function mouseMoveHandler(e: MouseEvent) {
				if (el === null || !is_dragging) {
					return;
				}
				let wrap_position = el.offsetLeft;
				// change this and other stuff to not use constants that aren't based on the specified dimensions of the component
				if (Math.abs(e.clientX - wrap_position) > 5) {
					position = e.clientX - offsetX;
					position = Math.max(Math.min(position, width - indicator_width), 0);
				} else {
					position = e.clientX - offsetX;
					position = Math.max(Math.min(position, width - indicator_width), 0);
				}
				indicator.style.left = position + "px";

				update_value();
			}
			function reset() {
				// have to call this here...maybe want to change how this is handled later
				is_dragging = false;
				update_database();

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

<div style="display: flex; flex-direction: column; gap: 4px;">
	<div
		class="wrapper"
		style="width:{width}px; height:{height}"
		bind:this={el}
		data-attribute={is_dragging}
		role="button"
		tabindex={-1}
		on:mousedown={(e) => {
			is_dragging = true;
			if (e.shiftKey) {
			}
			let wrap_position = el.offsetLeft;
			if (Math.abs(e.clientX - wrap_position) > 5) {
				position = e.clientX - wrap_position - 5;
				position = Math.max(Math.min(position, width - indicator_width), 0);
			} else {
				position = e.clientY - wrap_position - 5;
				position = Math.max(Math.min(position, width - indicator_width), 0);
			}
			indicator.style.left = position + "px";
			update_value();
		}}
		on:mouseup={() => {
			is_dragging = false;
		}}
	>
		<div
			class="thumb"
			style="width:{indicator_width}px; height:{height}"
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
		<span class="value-text">{value.toFixed(0)}</span>
	</div>
	<div>
		<div
			class="scroll-number"
			style="float: left;"
			on:wheel={(e) => {
				e.preventDefault();
				// mouse scroll has different deltaY than trackpad
				if (e.deltaY > 5) {
					let v = e.deltaY + control_min_freq;
					control_min_freq = Math.max(0, Math.min(v, value));
				} else {
					let v = e.deltaY * 5 + control_min_freq;
					control_min_freq = Math.max(0, Math.min(v, value));
				}
				update_position();
			}}
		>
			{control_min_freq.toFixed(0)}
		</div>
		<div
			class="scroll-number"
			style="float: right;"
			on:wheel={(e) => {
				e.preventDefault();
				if (e.deltaY > 5) {
					let v = e.deltaY + control_max_freq;
					control_max_freq = Math.max(value, Math.min(v, NYQUIST));
				} else {
					let v = e.deltaY * 10 + control_max_freq;
					control_max_freq = Math.max(value, Math.min(v, NYQUIST));
				}
				update_position();
			}}
		>
			{control_max_freq.toFixed(0)}
		</div>
	</div>
</div>

<style>
	.wrapper {
		background: var(--slider-background);
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
		background: var(--slider-indicator);
		position: absolute;
	}
	.thumb:active {
		background: var(--slider-active);
	}
	.thumb[data-attribute="true"] {
		background: var(--slider-active);
	}
	.value-text {
		font-size: 12px;
		pointer-events: none;
		font-weight: bold;
		z-index: 1;
	}
	.scroll-number {
		background: var(--slider-background);
		border: 1px solid black;
		transition: border-color 0.33s;
		cursor: ns-resize;
		width: 3em;
		font-size: 12px;
	}
	.scroll-number:hover {
		border-color: var(--slider-hover);
	}
</style>
