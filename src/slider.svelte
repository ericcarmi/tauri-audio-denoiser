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

<div class="wrapper">
	<div class="thumb" bind:this={el}/>
</div>

<style>
	.wrapper {
		background: black;
		width: 2.5em;
		display: flex;
		justify-content: center;
		border: 1px solid var(--purple);
		position: relative;
	}
	.thumb {
		background: var(--orange);
		width: 80%;
		height: 1em;
		position: absolute;
	}
</style>
