<script lang="ts">
	import { onMount } from "svelte";
	import { MAX_FREQ, MIN_FREQ, NYQUIST } from "./constants.svelte";

	export let value: number;
	export let index: number;
	export let update_server = () => {};

	let coarse_position = 0;
	let coarse_el: HTMLElement;
	let coarse_indicator: HTMLElement;
	let coarse_is_dragging = false;
	let coarse_value = value;

	let fine_position = 0;
	let fine_el: HTMLElement;
	let fine_indicator: HTMLElement;
	let fine_is_dragging = false;
	let fine_value = 0;

	let width = 180;
	let indicator_width = 5;

	let control_max_freq = NYQUIST;
	let control_min_freq = 20;

	let high_res = 0.1;
	let low_res = 0.5;
	let resolution = 1;

	function update_coarse_value() {
		let x =
			((coarse_position - 0) / width) * (control_max_freq - control_min_freq) +
			control_min_freq;
		// let logfreq = linlog(x, control_min_freq, control_max_freq)
		coarse_value = x;
		value = coarse_value + fine_value;
		// might want to reset fine back to 0...or do that after letting go of fine? it snaps back...
		// fine_value = 0;
		// fine_position = width/2
	}

	function update_fine_value() {
		let x = (fine_position / width) * (width + indicator_width - -width) * 0.5;
		fine_value = x - 100;
		value = coarse_value + fine_value;
	}

	function coarse_draggable() {
		if (coarse_el === null) {
			return;
		}
		coarse_indicator.addEventListener("mousedown", function (e: MouseEvent) {
			var offsetX = e.clientX - coarse_position;
			function mouseMoveHandler(e: MouseEvent) {
				if (coarse_el === null || !coarse_is_dragging) {
					return;
				}

				coarse_position = e.clientX - offsetX;
				coarse_position = Math.max(
					Math.min(coarse_position, width - indicator_width),
					0
				);
				coarse_indicator.style.left = coarse_position + "px";
				update_coarse_value();
			}
			function reset() {
				coarse_is_dragging = false;
				update_server();
				window.removeEventListener("mousemove", mouseMoveHandler);
				window.removeEventListener("mouseup", reset);
			}
			window.addEventListener("mousemove", mouseMoveHandler);
			window.addEventListener("mouseup", reset);
		});
		coarse_el.addEventListener("mousedown", function (e: MouseEvent) {
			var offsetX = e.clientX - coarse_position;

			function mouseMoveHandler(e: MouseEvent) {
				if (coarse_el === null || !coarse_is_dragging) {
					return;
				}
				let wrap_position = coarse_el.offsetLeft;
				// change this and other stuff to not use constants that aren't based on the specified dimensions of the component
				if (Math.abs(e.clientX - wrap_position) > 5) {
					coarse_position = e.clientX - offsetX;
					coarse_position = Math.max(
						Math.min(coarse_position, width - indicator_width),
						0
					);
				} else {
					coarse_position = e.clientX - offsetX;
					coarse_position = Math.max(
						Math.min(coarse_position, width - indicator_width),
						0
					);
				}
				coarse_indicator.style.left = coarse_position + "px";

				update_coarse_value();
			}
			function reset() {
				// have to call this here...maybe want to change how this is handled later
				coarse_is_dragging = false;
				update_server();

				window.removeEventListener("mousemove", mouseMoveHandler);
				window.removeEventListener("mouseup", reset);
			}
			window.addEventListener("mousemove", mouseMoveHandler);
			window.addEventListener("mouseup", reset);
		});
	}

	function fine_draggable() {
		if (fine_el === null) {
			return;
		}
		fine_indicator.addEventListener("mousedown", function (e: MouseEvent) {
			var offsetX = e.clientX - fine_position;
			function mouseMoveHandler(e: MouseEvent) {
				if (fine_el === null || !fine_is_dragging) {
					return;
				}

				fine_position = e.clientX - offsetX;
				fine_position = Math.max(
					Math.min(fine_position, width - indicator_width),
					0
				);
				fine_indicator.style.left = fine_position + "px";
				update_fine_value();
			}
			function reset() {
				fine_is_dragging = false;
				// fine_value = 10;
				// fine_position = width / 2 + indicator_width;
				// fine_indicator.style.left = fine_position + "px";
				// update_fine_value();
				update_server();
				window.removeEventListener("mousemove", mouseMoveHandler);
				window.removeEventListener("mouseup", reset);
			}
			window.addEventListener("mousemove", mouseMoveHandler);
			window.addEventListener("mouseup", reset);
		});
		fine_el.addEventListener("mousedown", function (e: MouseEvent) {
			var offsetX = e.clientX - fine_position;

			function mouseMoveHandler(e: MouseEvent) {
				if (fine_el === null || !fine_is_dragging) {
					return;
				}
				let wrap_position = fine_el.offsetLeft;
				// change this and other stuff to not use constants that aren't based on the specified dimensions of the component
				if (Math.abs(e.clientX - wrap_position) > 5) {
					fine_position = e.clientX - offsetX;
					fine_position = Math.max(
						Math.min(fine_position, width - indicator_width),
						0
					);
				} else {
					fine_position = e.clientX - offsetX;
					fine_position = Math.max(
						Math.min(fine_position, width - indicator_width),
						0
					);
				}
				fine_indicator.style.left = fine_position + "px";

				update_fine_value();
			}
			function reset() {
				// have to call this here...maybe want to change how this is handled later
				// fine_value = 10;
				// fine_position = width / 2 + indicator_width;
				// fine_indicator.style.left = fine_position + "px";
				// update_fine_value();
				fine_is_dragging = false;
				update_server();

				window.removeEventListener("mousemove", mouseMoveHandler);
				window.removeEventListener("mouseup", reset);
			}
			window.addEventListener("mousemove", mouseMoveHandler);
			window.addEventListener("mouseup", reset);
		});
	}

	onMount(() => {
		coarse_draggable();
		fine_draggable();
	});
</script>

<div style="display: flex; flex-direction: column; gap: 4px;">
	<div
		class="wrapper"
		style="width:{width}px"
		bind:this={coarse_el}
		data-attribute={coarse_is_dragging}
		role="button"
		tabindex={-1}
		on:mousedown={(e) => {
			coarse_is_dragging = true;
			if (e.shiftKey) {
				resolution = high_res;
			}
			let wrap_position = coarse_el.offsetLeft;
			if (Math.abs(e.clientX - wrap_position) > 5) {
				coarse_position = e.clientX - wrap_position - 5;
				coarse_position = Math.max(
					Math.min(coarse_position, width - indicator_width),
					0
				);
			} else {
				coarse_position = e.clientY - wrap_position - 5;
				coarse_position = Math.max(
					Math.min(coarse_position, width - indicator_width),
					0
				);
			}
			coarse_indicator.style.left = coarse_position + "px";
			update_coarse_value();
		}}
		on:mouseup={() => {
			coarse_is_dragging = false;
			resolution = low_res;
		}}
	>
		<div
			class="thumb"
			style="width:{indicator_width}px"
			bind:this={coarse_indicator}
			data-attribute={coarse_is_dragging}
			role="button"
			tabindex={-1}
			on:mousedown={(e) => {
				if (e.shiftKey) {
					resolution = high_res;
				}
				coarse_is_dragging = true;
			}}
			on:mouseup={() => {
				coarse_is_dragging = false;
				resolution = low_res;
			}}
		/>
		<span class="value-text">{value.toFixed(1)}</span>
	</div>

	<div
		class="wrapper"
		style="width:{width}px"
		bind:this={fine_el}
		data-attribute={fine_is_dragging}
		role="button"
		tabindex={-1}
		on:mousedown={(e) => {
			fine_is_dragging = true;
			if (e.shiftKey) {
				resolution = high_res;
			}
			let wrap_position = fine_el.offsetLeft;
			if (Math.abs(e.clientX - wrap_position) > 5) {
				fine_position = e.clientX - wrap_position - 5;
				fine_position = Math.max(
					Math.min(fine_position, width - indicator_width),
					0
				);
			} else {
				fine_position = e.clientY - wrap_position - 5;
				fine_position = Math.max(
					Math.min(fine_position, width - indicator_width),
					0
				);
			}
			fine_indicator.style.left = fine_position + "px";
			update_fine_value();
		}}
		on:mouseup={() => {
			fine_is_dragging = false;
			resolution = low_res;
		}}
	>
		<div
			class="thumb"
			style="width:{indicator_width}px"
			bind:this={fine_indicator}
			data-attribute={fine_is_dragging}
			role="button"
			tabindex={-1}
			on:mousedown={(e) => {
				if (e.shiftKey) {
					resolution = high_res;
				}
				fine_is_dragging = true;
			}}
			on:mouseup={() => {
				fine_is_dragging = false;
				resolution = low_res;
			}}
		/>
		<span class="value-text">{fine_value.toFixed(1)}</span>
	</div>
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
