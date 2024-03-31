<script lang="ts">
	import { onMount } from "svelte";

	export let value;
	// probably don't need to export this, just call invoke/update_filters from here

	let position = 0;
	let el: any;

	function draggable() {
		if (el === null) {
			return;
		}
		el.addEventListener("mousedown", function (e: MouseEvent) {
			// var offsetX = e.clientX - 10;
			var offsetY = e.clientY - position;

			// offsetY = Math.max(offsetY, 0);

			function mouseMoveHandler(e: any) {
				if (el === null) {
					return;
				}
				position = e.clientY - offsetY;
				position = Math.max(Math.min(position, 80), 0);
				el.style.top = position + "px";
				// el.style.left = e.clientX - offsetX + "px";
				value = position / 80;
				// could be an issue with only so much resolution? maybe not, idk...i wonder of regular inputs fake the range? those can increase number of values in between somehow
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
	}
	onMount(() => {
		draggable();
	});

	let is_dragging = false;
	let is_hovering = false;

</script>

<div
	class="wrapper"
	data-attribute={is_dragging}
	role="button"
	tabindex={-1}
		on:mouseenter={() => {
			is_hovering = true;
		}}
		on:mouseleave={() => {
			is_hovering = false;
		}}
>
	<div
		class="thumb"
		bind:this={el}
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
		border: 1px solid var(--orange);
	}

	.wrapper[data-attribute="true"] {
		border: 1px solid var(--orange);
	}

	.thumb {
		background: black;
		width: 90%;
		height: 1em;
		position: absolute;
	}

	.thumb:active {
		background: var(--orange);
	}
</style>
