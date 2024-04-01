<script context="module" lang="ts">
	import { FREQ_PLOT_WIDTH } from "./constants.svelte";

	export type FilterBank = Record<string, Array<number>>;

	export type Complex = {re: number, im: number};

	export type BPF = {
		gain: number,
		freq: number,
		Q: number,
	}

	export type FilterCoeffs2 = {
		b0: number,
		b1: number,
		b2: number,
		a0: number,
		a1: number,
		a2: number,
	}

	export interface Recording {
		created: string;
		uid: number;
	}
	export interface Folder {
		folder_name: string;
		files: Array<any>;
		showing: boolean;
	}

</script>
