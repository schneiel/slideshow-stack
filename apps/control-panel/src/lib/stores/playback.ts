import { writable, derived } from 'svelte/store';
import { playbackApi, type ScalingMode } from '$lib/api/playback';
import type {
	StopSlideshowRequest,
	PauseRequest,
	ResumeRequest,
	NextImageRequest,
	PreviousImageRequest,
	SetScalingRequest
} from '$lib/api/playback';

export interface PlaybackState {
	isPlaying: boolean;
	isPaused: boolean;
	slideshowName: string;
	currentImage: string;
	imageIndex: number;
	totalImages: number;
	progress: number;
	currentDeviceId?: string;
	errorMessage?: string;
}

const createPlaybackStore = () => {
	const { subscribe, set, update } = writable<PlaybackState>({
		isPlaying: false,
		isPaused: false,
		slideshowName: 'None',
		currentImage: 'None',
		imageIndex: 0,
		totalImages: 0,
		progress: 0
	});

	return {
		subscribe,
		set,
		update,

		startSlideshow: async (slideshowId: string, targetDeviceIds?: string[]) => {
			try {
				update((state) => {
					const result = { ...state };
					delete result.errorMessage;
					return result;
				});

				const { slideshowApi } = await import('$lib/api/slideshow');
				const slideshow = await slideshowApi.getSlideshow(slideshowId);

				if (!slideshow) {
					throw new Error('Slideshow not found');
				}

				const request = playbackApi.buildStartSlideshowRequest(slideshow, targetDeviceIds);

				await playbackApi.startSlideshow(request);

				update((state) => {
					const result = { ...state };
					delete result.errorMessage;
					return result;
				});
			} catch (error) {
				const message = error instanceof Error ? error.message : 'Failed to start slideshow';
				update((state) => ({
					...state,
					errorMessage: message
				}));
				throw error;
			}
		},

		stopSlideshow: async (targetDeviceIds?: string[], graceful = true) => {
			try {
				update((state) => {
					const result = { ...state };
					delete result.errorMessage;
					return result;
				});

				const request: StopSlideshowRequest = {
					graceful
				};
				if (targetDeviceIds) {
					request.target_device_ids = targetDeviceIds;
				}

				await playbackApi.stopSlideshow(request);

				update((state) => {
					const result = { ...state };
					delete result.errorMessage;
					return result;
				});
			} catch (error) {
				const message = error instanceof Error ? error.message : 'Failed to stop slideshow';
				update((state) => ({
					...state,
					errorMessage: message
				}));
				throw error;
			}
		},

		pauseSlideshow: async (targetDeviceIds?: string[]) => {
			try {
				update((state) => {
					const result = { ...state };
					delete result.errorMessage;
					return result;
				});

				const request: PauseRequest = {};
				if (targetDeviceIds) {
					request.target_device_ids = targetDeviceIds;
				}

				await playbackApi.pauseSlideshow(request);

				update((state) => {
					const result = { ...state };
					delete result.errorMessage;
					return result;
				});
			} catch (error) {
				const message = error instanceof Error ? error.message : 'Failed to pause slideshow';
				update((state) => ({
					...state,
					errorMessage: message
				}));
				throw error;
			}
		},

		resumeSlideshow: async (targetDeviceIds?: string[]) => {
			try {
				update((state) => {
					const result = { ...state };
					delete result.errorMessage;
					return result;
				});

				const request: ResumeRequest = {};
				if (targetDeviceIds) {
					request.target_device_ids = targetDeviceIds;
				}

				await playbackApi.resumeSlideshow(request);

				update((state) => {
					const result = { ...state };
					delete result.errorMessage;
					return result;
				});
			} catch (error) {
				const message = error instanceof Error ? error.message : 'Failed to resume slideshow';
				update((state) => ({
					...state,
					errorMessage: message
				}));
				throw error;
			}
		},

		nextImage: async (targetDeviceIds?: string[]) => {
			try {
				update((state) => {
					const result = { ...state };
					delete result.errorMessage;
					return result;
				});

				const request: NextImageRequest = {};
				if (targetDeviceIds) {
					request.target_device_ids = targetDeviceIds;
				}

				await playbackApi.nextImage(request);

				update((state) => {
					const result = { ...state };
					delete result.errorMessage;
					return result;
				});
			} catch (error) {
				const message = error instanceof Error ? error.message : 'Failed to go to next image';
				update((state) => ({
					...state,
					errorMessage: message
				}));
				throw error;
			}
		},

		previousImage: async (targetDeviceIds?: string[]) => {
			try {
				update((state) => {
					const result = { ...state };
					delete result.errorMessage;
					return result;
				});

				const request: PreviousImageRequest = {};
				if (targetDeviceIds) {
					request.target_device_ids = targetDeviceIds;
				}

				await playbackApi.previousImage(request);

				update((state) => {
					const result = { ...state };
					delete result.errorMessage;
					return result;
				});
			} catch (error) {
				const message = error instanceof Error ? error.message : 'Failed to go to previous image';
				update((state) => ({
					...state,
					errorMessage: message
				}));
				throw error;
			}
		},

		setScaling: async (mode: number, targetDeviceIds?: string[]) => {
			try {
				update((state) => {
					const result = { ...state };
					delete result.errorMessage;
					return result;
				});

				const request: SetScalingRequest = { mode };
				if (targetDeviceIds) {
					request.target_device_ids = targetDeviceIds;
				}

				await playbackApi.setScaling(request);

				update((state) => {
					const result = { ...state };
					delete result.errorMessage;
					return result;
				});
			} catch (error) {
				const message = error instanceof Error ? error.message : 'Failed to cycle scaling mode';
				update((state) => ({
					...state,
					errorMessage: message
				}));
				throw error;
			}
		},

		clearError: () => {
			update((state) => {
				const result = { ...state };
				delete result.errorMessage;
				return result;
			});
		},

		disconnect: () => {}
	};
};

export const playbackStore = createPlaybackStore();

export const isPlaying = derived(playbackStore, (state) => state.isPlaying);
export const isPaused = derived(playbackStore, (state) => state.isPaused);
export const slideshowName = derived(playbackStore, (state) => state.slideshowName);
export const currentImage = derived(playbackStore, (state) => state.currentImage);
export const imageIndex = derived(playbackStore, (state) => state.imageIndex);
export const totalImages = derived(playbackStore, (state) => state.totalImages);
export const progress = derived(playbackStore, (state) => state.progress);
export const errorMessage = derived(playbackStore, (state) => state.errorMessage);

export const {
	startSlideshow,
	stopSlideshow,
	pauseSlideshow,
	resumeSlideshow,
	nextImage,
	previousImage,
	setScaling,
	clearError
} = playbackStore;

export interface SlideshowConfig {
	name: string;
	media_ids: string[];
	interval_seconds: number;
	shuffle: boolean;
	loop_enabled: boolean;
	scaling_mode?: ScalingMode;
}

import { showError, showInfo } from '$lib/stores/toast';

/**
 * Start a slideshow with a given configuration on selected devices.
 * This is the unified function used by all slideshow start operations.
 */
export async function startSlideshowWithConfig(
	config: SlideshowConfig,
	targetDeviceIds: string[]
): Promise<boolean> {
	try {
		const options: {
			scaling_mode?: ScalingMode;
		} = {};
		if (config.scaling_mode) {
			options.scaling_mode = config.scaling_mode;
		}

		const request = playbackApi.buildStartSlideshowRequest(
			{
				name: config.name,
				media_ids: config.media_ids,
				interval_seconds: config.interval_seconds,
				shuffle: config.shuffle,
				loop_enabled: config.loop_enabled
			},
			targetDeviceIds,
			options
		);

		await playbackApi.startSlideshow(request);

		showInfo(`Request to start slideshow sent`, 2000);
		return true;
	} catch (e) {
		const errorMessage = e instanceof Error ? e.message : 'Unknown error';
		showError(`Failed to send start command: ${errorMessage}`);
		return false;
	}
}

/**
 * Start a saved slideshow by ID.
 */
export async function startSavedSlideshow(
	slideshowId: string,
	targetDeviceIds: string[]
): Promise<boolean> {
	try {
		const { slideshowApi } = await import('$lib/api/slideshow');
		const slideshow = await slideshowApi.getSlideshow(slideshowId);
		if (!slideshow) {
			showError('Slideshow not found');
			return false;
		}

		return startSlideshowWithConfig(
			{
				name: slideshow.name,
				media_ids: slideshow.media_ids,
				interval_seconds: slideshow.interval_seconds,
				shuffle: slideshow.shuffle || false,
				loop_enabled: slideshow.loop_enabled,
				scaling_mode: 'fit_to_screen' as ScalingMode
			},
			targetDeviceIds
		);
	} catch (e) {
		const errorMessage = e instanceof Error ? e.message : 'Unknown error';
		showError(`Failed to start slideshow: ${errorMessage}`);
		return false;
	}
}

/**
 * Start a single image as a slideshow.
 */
export async function startSingleImage(
	filename: string,
	targetDeviceIds: string[]
): Promise<boolean> {
	return startSlideshowWithConfig(
		{
			name: filename,
			media_ids: [filename],
			interval_seconds: 5,
			shuffle: false,
			loop_enabled: true,
			scaling_mode: 'fit_to_screen' as ScalingMode
		},
		targetDeviceIds
	);
}
