<script lang="ts">
	import { onMount } from "svelte";

	export let value: number = 28.8;
	export let index: number;
	export let update_backend = () => {};
	export let update_database = () => {};

	let position = 0;
	let el: HTMLElement;
	let indicator: HTMLElement;

	let height = 52;
	let is_dragging = false;
	let range = 60;

	$: value, indicator && el && redraw();

	function redraw() {
		position = height / 2 - (value / range) * height;
		indicator.style.top = position + "px";
		update_backend()
	}

	function draggable() {
		if (el === null) {
			return;
		}
		indicator.addEventListener("mousedown", function (e: MouseEvent) {
			var offsetY = e.clientY - position;
			function mouseMoveHandler(e: any) {
				if (el === null || !is_dragging) {
					return;
				}
				position = e.clientY - offsetY;
				position = Math.max(Math.min(position, height), 1);
				indicator.style.top = position + "px";
				value = (0.5 - position / height) * range;
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
		el.addEventListener("mousedown", function (e: MouseEvent) {
			var offsetY = e.clientY - position;

			function mouseMoveHandler(e: any) {
				if (el === null || !is_dragging) {
					return;
				}
				let wrap_position = el.offsetTop;
				// change this and other stuff to not use constants that aren't based on the specified dimensions of the component
				if (Math.abs(e.clientY - wrap_position) > 5) {
					position = e.clientY - wrap_position - 5;
					position = Math.max(Math.min(position, height), 1);
				} else {
					position = e.clientY - offsetY;
					position = Math.max(Math.min(position, height), 1);
				}
				indicator.style.top = position + "px";
				value = (0.5 - position / height) * range;
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

<div
	class="wrapper"
	bind:this={el}
	data-attribute={is_dragging}
	role="button"
	tabindex={0}
	on:mousedown={(e) => {
		is_dragging = true;
		let wrap_position = el.offsetTop;
		if (Math.abs(e.clientY - wrap_position) > 5) {
			position = e.clientY - wrap_position - 5;
			position = Math.max(Math.min(position, height), 1);
		}
		indicator.style.top = position + "px";
		value = (0.5 - position / height) * range;
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
		position: absolute;
		z-index: 1;
		top: 30%;
		font-size: 12px;
		color: var(--gray200);
		pointer-events: none;
	}
	.wrapper {
		background: var(--gray100);
		width: 2.5em;
		display: flex;
		justify-content: center;
		border: 1px solid var(--slider-border);
		position: relative;
		transition: border 0.33s;
		height: 100%;
	}

	.wrapper:hover {
		border: 1px solid var(--slider-hover);
	}

	.wrapper[data-attribute="true"] {
		border: 1px solid var(--slider-hover);
	}

	.thumb {
		background: black;
		width: 90%;
		height: 0.5em;
		position: absolute;
		top: 1px;
	}

	.thumb:active {
		background: var(--slider-active);
	}

	.thumb[data-attribute="true"] {
		background: var(--slider-active);
	}
</style>
