<script lang="ts">
	import { onMount } from "svelte";

	export let value;
	let position = 0;
	let el: HTMLElement;
	let indicator: HTMLElement;

	let height = 52;
	let is_dragging = false;
	let is_hovering = false;

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
				value = (0.5 - position / height) * 30;
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
				value = (0.5 - position / height) * 30;
				console.log(position, wrap_position, e.clientY);
			}
			function reset() {
				// have to call this here...maybe want to change how this is handled later
				console.log("reset");

				is_dragging = false;
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
	on:mouseenter={() => {
		is_hovering = true;
	}}
	on:mouseleave={() => {
		is_hovering = false;
	}}
	on:mousedown={() => {
		is_dragging = true;
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
</div>

<style>
	.wrapper {
		background: var(--gray100);
		width: 2.5em;
		display: flex;
		justify-content: center;
		border: 1px solid var(--purple);
		position: relative;
		transition: border 0.33s;
	}

	.wrapper:hover {
		border: 1px solid var(--lightpurple);
	}

	.wrapper[data-attribute="true"] {
		border: 1px solid var(--lightpurple);
	}

	.thumb {
		background: black;
		width: 90%;
		height: 0.5em;
		position: absolute;
		top: 1px;
	}

	.thumb:active {
		background: var(--purple);
	}

	.thumb[data-attribute="true"] {
		background: var(--purple);
	}
</style>
