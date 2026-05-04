import { API_ENDPOINTS } from '$lib/api/config';

export interface StartSlideshowRequest {
	target_device_ids?: string[];
	name?: string;
	media_files: string[];
	interval_seconds: number;
	shuffle_enabled: boolean;
	scaling_mode?: string;
	loop_enabled: boolean;
}

export interface StopSlideshowRequest {
	target_device_ids?: string[];
	graceful: boolean;
}

export interface PauseRequest {
	target_device_ids?: string[];
}

export interface ResumeRequest {
	target_device_ids?: string[];
}

export interface NextImageRequest {
	target_device_ids?: string[];
}

export interface PreviousImageRequest {
	target_device_ids?: string[];
}

export interface SetScalingRequest {
	target_device_ids?: string[];
	mode: number;
}

export interface StartVideoRequest {
	target_device_ids?: string[];
	filename: string;
}

export interface VideoControlRequest {
	target_device_ids?: string[];
}

export interface UpdateAutostartConfigRequest {
	target_device_ids?: string[];
	autostart_config: {
		enabled: boolean;
		name?: string;
		type?: string;
		media_files?: string[];
		interval_seconds?: number;
		shuffle_enabled?: boolean;
		scaling_mode?: string;
		loop_enabled?: boolean;
	};
}

export interface ClientInfo {
	device_id: string;
	display_name: string;
	client_type: string;
	status: unknown;
	last_activity: string;
}

export interface EventData {
	timestamp: string;
	device_id: string;
	event: unknown;
}

export const SCALING_MODES = {
	ONE_TO_ONE: '1:1',
	STRETCH: 'stretch_to_fit',
	FILL: 'fill_to_screen',
	FIT: 'fit_to_screen'
} as const;

export type ScalingMode = (typeof SCALING_MODES)[keyof typeof SCALING_MODES];

class PlaybackAPIClient {
	private eventSource: EventSource | null = null;
	private readonly basePath = API_ENDPOINTS.PLAYBACK.API;

	constructor() {}

	private initializeEventSource(): void {
		try {
			if (this.eventSource) {
				this.eventSource.close();
			}

			const eventsUrl = `${API_ENDPOINTS.PLAYBACK.STREAM}/events`;
			this.eventSource = new EventSource(eventsUrl);

			this.eventSource.onmessage = (event) => {
				try {
					const eventData = JSON.parse(event.data);
					this.handleEvent(eventData);
				} catch {
					// Ignore malformed event data
				}
			};

			this.eventSource.onerror = () => {
				this.eventSource = null;
			};
		} catch {
			// EventSource initialization failed - will retry on next call
		}
	}

	handleEvent(eventData: unknown): void {
		for (const handler of this.eventHandlers) {
			try {
				handler(eventData as EventData);
			} catch {
				// Handler failed - continue with other handlers
			}
		}
	}

	private eventHandlers: Set<(eventData: EventData) => void> = new Set();

	addEventHandler(handler: (eventData: EventData) => void): void {
		this.eventHandlers.add(handler);
	}

	removeEventHandler(handler: (eventData: EventData) => void): void {
		this.eventHandlers.delete(handler);
	}

	private async executeCommand(
		endpoint: string,
		body?: unknown
	): Promise<{ status: string; message: string }> {
		await this.sendCommand(endpoint, body);
		return {
			status: 'accepted',
			message: 'Command sent (fire-and-forget)'
		};
	}

	private async sendCommand(endpoint: string, body?: unknown): Promise<void> {
		const url = `${this.basePath}${endpoint}`;

		const options: RequestInit = {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			}
		};

		if (body) {
			options.body = JSON.stringify(body);
		}

		const response = await fetch(url, options);

		if (!response.ok) {
			throw new Error(`Server returned status ${response.status}`);
		}
	}

	async startSlideshow(
		request: StartSlideshowRequest
	): Promise<{ status: string; message: string }> {
		return this.executeCommand('/slideshow/start', request);
	}

	async stopSlideshow(request: StopSlideshowRequest): Promise<{ status: string; message: string }> {
		return this.executeCommand('/slideshow/stop', request);
	}

	async pauseSlideshow(request: PauseRequest): Promise<{ status: string; message: string }> {
		return this.executeCommand('/slideshow/pause', request);
	}

	async resumeSlideshow(request: ResumeRequest): Promise<{ status: string; message: string }> {
		return this.executeCommand('/slideshow/resume', request);
	}

	async nextImage(request: NextImageRequest): Promise<{ status: string; message: string }> {
		return this.executeCommand('/slideshow/next', request);
	}

	async previousImage(request: PreviousImageRequest): Promise<{ status: string; message: string }> {
		return this.executeCommand('/slideshow/previous', request);
	}

	async setScaling(request: SetScalingRequest): Promise<{ status: string; message: string }> {
		return this.executeCommand('/slideshow/set-scaling', request);
	}

	async updateAutostartConfig(
		request: UpdateAutostartConfigRequest
	): Promise<{ status: string; message: string }> {
		return this.executeCommand('/slideshow/update-autostart', request);
	}

	async startVideo(request: StartVideoRequest): Promise<{ status: string; message: string }> {
		return this.executeCommand('/video/start', request);
	}

	async stopVideo(request: VideoControlRequest): Promise<{ status: string; message: string }> {
		return this.executeCommand('/video/stop', request);
	}

	async pauseVideo(request: VideoControlRequest): Promise<{ status: string; message: string }> {
		return this.executeCommand('/video/pause', request);
	}

	async resumeVideo(request: VideoControlRequest): Promise<{ status: string; message: string }> {
		return this.executeCommand('/video/resume', request);
	}

	async setVideoScaling(request: SetScalingRequest): Promise<{ status: string; message: string }> {
		return this.executeCommand('/video/set-scaling', request);
	}

	getEventStream(): EventSource | null {
		if (!this.eventSource) {
			this.initializeEventSource();
		}
		return this.eventSource;
	}

	disconnect(): void {
		if (this.eventSource) {
			this.eventSource.close();
			this.eventSource = null;
		}
	}

	buildStartSlideshowRequest(
		slideshow: {
			name: string;
			media_ids: string[];
			interval_seconds: number;
			shuffle: boolean;
			loop_enabled: boolean;
		},
		targetDeviceIds?: string[],
		options?: {
			scaling_mode?: ScalingMode;
		}
	): StartSlideshowRequest {
		const request: StartSlideshowRequest = {
			name: slideshow.name,
			media_files: slideshow.media_ids,
			interval_seconds: slideshow.interval_seconds,
			shuffle_enabled: slideshow.shuffle,
			scaling_mode: options?.scaling_mode || SCALING_MODES.FIT,
			loop_enabled: slideshow.loop_enabled
		};
		if (targetDeviceIds) {
			request.target_device_ids = targetDeviceIds;
		}
		return request;
	}
}

export const playbackApi = new PlaybackAPIClient();
export { PlaybackAPIClient };
