<script context="module" lang="ts">
	export const num_sliders = 5;
	export const FREQ_PLOT_WIDTH = 700;
	export const FREQ_PLOT_HEIGHT = 300;
	export const TIME_PLOT_WIDTH = 700;
	export const TIME_PLOT_HEIGHT = 50;
	export const SAMPLING_RATE = 44100;
	import type { Complex, FilterCoeffs2 } from "./types.svelte";

	export function biquad(gain: number, freq: number, Q: number) {
		let db = 20 * Math.log10(gain + 1e-6);
		let A = Math.pow(10, db / 40);
		let w0 = (2 * Math.PI * freq) / SAMPLING_RATE;
		let alpha = Math.sin(w0) / 2 / Q;

		return {
			b0: 1 + alpha * A,
			b1: -2 * Math.cos(w0),
			b2: 1 - alpha * A,
			a0: 1 + alpha / A,
			a1: -2 * Math.cos(w0),
			a2: 1 - alpha / A,
		} as FilterCoeffs2;
	}

	export function freq_response(coeffs: FilterCoeffs2, len: number) {
		let z0: Array<Complex> = [];
		let z1: Array<Complex> = [];
		let z2: Array<Complex> = [];
		let H: Array<number> = [];
		for (let i = 1; i < len; i++) {
			let w = ((2 * Math.PI * i) / len);
			let x = Math.cos(w);
			let y = Math.sin(w);
			let c: Complex = { re: x, im: y };
			let c1: Complex = { re: x * x, im: y * y  };
			let c2: Complex = { re: x * x * x, im: y * y * y };
			z0.push(c);
			z1.push(c1);
			z2.push(c2);

			let hre =
				(coeffs.b0 * c.re + coeffs.b1 * c1.re + coeffs.b2 * c2.re) /
				(coeffs.a0 * c.re + coeffs.a1 * c1.re + coeffs.a2 * c2.re);
			let him =
				(coeffs.b0 * c.im + coeffs.b1 * c1.im + coeffs.b2 * c2.im) /
				(coeffs.a0 * c.im + coeffs.a1 * c1.im + coeffs.a2 * c2.im);

			H.push(Math.abs(hre * hre + him * him));
		}

		return H;
	}
</script>
