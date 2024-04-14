<script lang="ts" context="module">
	import { invoke } from "@tauri-apps/api/tauri";
	import { FREQ_PLOT_WIDTH, SAMPLING_RATE } from "./constants.svelte";
	import type {
		BPF,
		ChannelParams,
		Complex,
		FilterCoeffs2,
		StereoControl,
		StereoParams,
	} from "./types.svelte";

	export function cabs(z: Complex) {
		return Math.sqrt(z.re * z.re + z.im * z.im);
	}

	export function init_channel_params(
		gains: number[],
		freqs: number[],
		Qs: number[]
	): StereoParams {
		if (gains.length !== freqs.length || gains.length !== Qs.length) {
			console.error("array length mismatch");
			// return;
		}
		let output_gain = 0.0;
		let noise_gain = 0.0;
		let pre_smooth_gain = 0.5;
		let post_smooth_gain = 0.5;
		let clean = false;
		let bpf_filters: Array<BPF> = Array(gains.length)
			.fill(0)
			.map((_, i) => {
				return { gain: gains[i], freq: freqs[i], Q: Qs[i] };
			});

		let c = {
			bpfs: bpf_filters,
			output_gain: output_gain,
			noise_gain: noise_gain,
			pre_smooth_gain: pre_smooth_gain,
			post_smooth_gain: post_smooth_gain,
			clean: clean,
			mute: false,
		} as ChannelParams;

		return {
			left: c,
			right: c,
			both: c,
			control: "Both",
			is_stereo: true,
			clean: false,
		};
	}

	export function update_css_color(color: string, color_name: string) {
		if (color !== undefined) {
			document.body.style.setProperty(
				`--${color_name.replace("_", "-")}`,
				color
			);
		}
	}

	export function cdiv(z1: Complex, z2: Complex) {
		let a = z1.re;
		let b = z1.im;
		let c = z2.re;
		let d = z2.im;
		return {
			re: (a * c + b * d) / (c ** 2 + d ** 2),
			im: (b * c - a * d) / (c ** 2 + d ** 2),
		} as Complex;
	}

	export function biquad(gain: number, freq: number, Q: number) {
		let A = Math.pow(10, gain / 40);
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
		let H: Array<number> = [];
		for (let i = 0; i < len; i++) {
			let w = (Math.PI * i) / len;
			let x = Math.cos(w);
			let y = Math.sin(w);
			let c1: Complex = { re: x, im: y };
			let c2: Complex = { re: x * x - y * y, im: 2 * x * y };

			let numre = coeffs.b0 + coeffs.b1 * c1.re + coeffs.b2 * c2.re;
			let numim = coeffs.b1 * c1.im + coeffs.b2 * c2.im;

			let denre = coeffs.a0 + coeffs.a1 * c1.re + coeffs.a2 * c2.re;
			let denim = coeffs.a1 * c1.im + coeffs.a2 * c2.im;

			let num: Complex = { re: numre, im: numim };
			let den: Complex = { re: denre, im: denim };

			H.push(20 * Math.log10(cabs(cdiv(num, den)) + 1e-6));
		}

		return H;
	}

	export function loglin(x: number, minfreq: number, maxfreq: number) {
		let y =
			((maxfreq - minfreq) * Math.log10(x / minfreq)) /
				Math.log10(maxfreq / minfreq) +
			minfreq;
		if (isNaN(y)) {
			console.error("result is nan for", x);
		}
		return y;
	}

	export function loglin2(x: number, minfreq: number, maxfreq: number) {
		let y =
			((maxfreq - minfreq) * Math.log2(x / minfreq)) /
				Math.log2(maxfreq / minfreq) +
			minfreq;
		if (isNaN(y)) {
			console.error("result is nan for", x);
		}
		return y;
	}

	export function linlog(x: number, minfreq: number, maxfreq: number) {
		return (
			minfreq *
			Math.pow(
				10,
				((x - minfreq) / (maxfreq - minfreq)) * Math.log10(maxfreq / minfreq)
			)
		);
	}

	export function linlog2(x: number, minfreq: number, maxfreq: number) {
		return (
			minfreq *
			Math.pow(
				2,
				((x - minfreq) / (maxfreq - minfreq)) * Math.log2(maxfreq / minfreq)
			)
		);
	}

	export function mel(x: number) {
		return 2595 * Math.log10(1 + x / 700);
	}
	export function bark_scale(x: number) {
		return 13 * Math.atan(0.00076 * x) + 3.5 * Math.atan((x / 7500) ** 2);
	}

	export const linspace = (start: number, stop: number, step: number) =>
		Array.from(
			{ length: (stop - start) / step + 1 },
			(_, i) => start + i * step
		);

	// this is probably not good
	export function setRectangle(
		gl: any,
		x: any,
		y: any,
		width: any,
		height: any
	) {
		var x1 = x;
		var x2 = x + width;
		var y1 = y;
		var y2 = y - height;
		gl.bufferData(
			gl.ARRAY_BUFFER,
			new Float32Array([x1, y1, x2, y1, x1, y2, x2, y2]),
			gl.STATIC_DRAW
		);
	}

	export function frequencyToXAxis(frequency: number) {
		const minF = Math.log(20) / Math.log(10);
		const maxF = Math.log(20000) / Math.log(10);

		let range = maxF - minF;
		let xAxis =
			((Math.log(frequency) / Math.log(10) - minF) / range) * FREQ_PLOT_WIDTH;
		return xAxis;
	}
	export function componentToHex(c: number) {
		var hex = c.toString(16);
		return hex.length == 1 ? "0" + hex : hex;
	}

	export function rgbToHex(rgb: any) {
		return (
			"#" +
			componentToHex(rgb.r) +
			componentToHex(rgb.g) +
			componentToHex(rgb.b)
		);
	}

	export function hexToRgb(hex: string) {
		var result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
		return result
			? {
					r: parseInt(result[1], 16),
					g: parseInt(result[2], 16),
					b: parseInt(result[3], 16),
			  }
			: null;
	}

	// for now just using this for the reset all gain switch, so the filters on backend are updated, and server at same time, which usually doesn't happen when moving gain slider (only sends to server on mouse up)
	export function update_filters(
		index: number,
		gain: number,
		freq: number,
		Q: number,
		update_server: boolean,
		stereo_control: StereoControl
	) {
		let b = biquad(gain, freq, Q) as any;
		b.x = [0, 0];
		b.y = [0, 0];
		// console.log('no')

		for (const key in b) {
			// console.log("what", b[key as keyof FilterCoeffs2]);

			// if (isNaN(b[key as keyof FilterCoeffs2])) return;
		}
		// console.log("here");
		if (index == 1) {
			invoke("update_filters", { bp1: b, stereoControl: stereo_control });
			if (update_server) {
				invoke("save_bpf_gain", {
					gain: gain,
					index: index,
					stereoControl: stereo_control,
				});
				invoke("save_bpf_freq", {
					freq: freq,
					index: index,
					stereoControl: stereo_control,
				});
				invoke("save_bpf_Q", {
					q: Q,
					index: index,
					stereoControl: stereo_control,
				});
			}
		} else if (index == 2) {
			invoke("update_filters", { bp2: b, stereoControl: stereo_control });
			if (update_server) {
				invoke("save_bpf_gain", {
					gain: gain,
					index: index,
					stereoControl: stereo_control,
				});
				invoke("save_bpf_freq", {
					freq: freq,
					index: index,
					stereoControl: stereo_control,
				});
				invoke("save_bpf_Q", {
					q: Q,
					index: index,
					stereoControl: stereo_control,
				});
			}
		} else if (index == 3) {
			invoke("update_filters", { bp3: b, stereoControl: stereo_control });
			if (update_server) {
				invoke("save_bpf_gain", {
					gain: gain,
					index: index,
					stereoControl: stereo_control,
				});
				invoke("save_bpf_freq", {
					freq: freq,
					index: index,
					stereoControl: stereo_control,
				});
				invoke("save_bpf_Q", {
					q: Q,
					index: index,
					stereoControl: stereo_control,
				});
			}
		} else if (index == 4) {
			invoke("update_filters", { bp4: b, stereoControl: stereo_control });
			if (update_server) {
				invoke("save_bpf_gain", {
					gain: gain,
					index: index,
					stereoControl: stereo_control,
				});
				invoke("save_bpf_freq", {
					freq: freq,
					index: index,
					stereoControl: stereo_control,
				});
				invoke("save_bpf_Q", {
					q: Q,
					index: index,
					stereoControl: stereo_control,
				});
			}
		} else if (index == 5) {
			invoke("update_filters", { bp5: b, stereoControl: stereo_control });
			if (update_server) {
				invoke("save_bpf_gain", {
					gain: gain,
					index: index,
					stereoControl: stereo_control,
				});
				invoke("save_bpf_freq", {
					freq: freq,
					index: index,
					stereoControl: stereo_control,
				});
				invoke("save_bpf_Q", {
					q: Q,
					index: index,
					stereoControl: stereo_control,
				});
			}
		}
	}

	export function remove_slashes_ext(s: string) {
		if (s.includes("/")) {
			let x = s.split("/");
			return x[x.length - 1].split(".")[0];
		} else {
			return s.split(".")[0];
		}
	}
</script>
