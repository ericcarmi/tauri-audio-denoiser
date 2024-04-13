<script context="module" lang="ts">
	export type Complex = { re: number; im: number };
	export type BPF = {
		gain: number;
		freq: number;
		Q: number;
	};


	export type ChannelParams = {
		bpfs: Array<BPF>,
		output_gain: number;
		noise_gain: number;
		pre_smooth_gain: number;
		post_smooth_gain: number;
		clean: boolean,
	}

	export type StereoParams = {
		left: ChannelParams,
		right: ChannelParams,
		both: ChannelParams,
		control: StereoControl,
		is_stereo: boolean,
		clean: boolean,
	}

	export type StereoControl = "Left" | "Right" | "Both";

	export type FilterBank = {
		bp1: FilterCoeffs2;
		bp2: FilterCoeffs2;
		bp3: FilterCoeffs2;
		bp4: FilterCoeffs2;
		bp5: FilterCoeffs2;
	};

	export type FilterCoeffs2 = {
		b0: number;
		b1: number;
		b2: number;
		a0: number;
		a1: number;
		a2: number;
		x: [number, number];
		y: [number, number];
	};

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
