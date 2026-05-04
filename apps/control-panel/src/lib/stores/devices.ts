import { writable, derived, get } from 'svelte/store';
import { playbackApi } from '$lib/api/playback';
import type { ClientInfo } from '$lib/api/playback';

interface DeviceStatus {
	healthy?: boolean;
	connected?: boolean;
	isRunning?: boolean;
	isPaused?: boolean;
	scalingMode?: string | undefined;
	currentImage?: string | undefined;
	imageIndex?: number | undefined;
	totalImages?: number | undefined;
	slideshowName?: string | undefined;
	interval?: number | undefined;
	windowSize?: { width: number; height: number } | undefined;
	lastUpdate?: Date | undefined;
}

interface VideoStatus {
	status?: string;
	filename?: string | undefined;
	currentFrame?: number | undefined;
	totalFrames?: number | undefined;
	isPlaying?: boolean;
	isPaused?: boolean;
	scalingMode?: string | undefined;
}

export interface Device {
	device_id: string;
	display_name: string;
	status?: DeviceStatus;
	videoStatus?: VideoStatus;
	isConnected: boolean;
	last_activity: string;
	healthy?: boolean;
}

export interface ActiveDevice {
	device_id: string | null;
	status?: DeviceStatus;
	isConnected: boolean;
	device?: Device;
}

interface SlideshowState {
	status: string;
	name?: string;
	image?: string;
	interval?: number;
	scaling_mode?: string;
	total_images?: number;
	current_index?: number;
}

interface VideoState {
	status: string;
	filename?: string;
	scaling_mode?: string;
}

interface DeviceState {
	device_id: string;
	display_name: string;
	connected: boolean;
	state: SlideshowState;
	video_state?: VideoState;
}

const createDevicesStore = () => {
	const devicesArray: Device[] = [];
	const { subscribe, set, update } = writable<Device[]>(devicesArray);

	function handleSSEEvent(eventData: unknown) {
		const { event_type, data } = eventData as {
			event_type: string;
			source?: string;
			data: unknown;
			device_id?: string;
		};

		if (event_type === 'slideshow_state' || event_type === 'video_state') {
			const deviceState = data as DeviceState;
			const stateData = deviceState.state;

			update((devices) => {
				const existingIndex = devices.findIndex((d) => d.device_id === deviceState.device_id);

				const displayName =
					deviceState.display_name || `Device ${deviceState.device_id.substring(0, 8)}`;
				const now = new Date().toISOString();

				const status: DeviceStatus = {
					lastUpdate: new Date(),
					connected: deviceState.connected,
					healthy: deviceState.connected,
					isRunning: stateData.status === 'playing',
					isPaused: stateData.status === 'paused',
					slideshowName: stateData.name || 'None',
					currentImage: stateData.image ? stateData.image.split('/').pop() : undefined,
					scalingMode: stateData.scaling_mode,
					interval: stateData.interval ?? undefined,
					imageIndex: stateData.current_index ?? undefined,
					totalImages: stateData.total_images ?? undefined
				};

				const videoStatus: VideoStatus | undefined = deviceState.video_state
					? {
							status: deviceState.video_state.status,
							filename: deviceState.video_state.filename
								? deviceState.video_state.filename.split('/').pop()
								: undefined,
							isPlaying: deviceState.video_state.status === 'playing',
							isPaused: deviceState.video_state.status === 'paused',
							scalingMode: deviceState.video_state.scaling_mode
						}
					: undefined;

				const updatedDevice: Device = {
					device_id: deviceState.device_id,
					display_name: displayName,
					status,
					isConnected: deviceState.connected,
					last_activity: now,
					healthy: deviceState.connected
				};

				if (videoStatus) {
					updatedDevice.videoStatus = videoStatus;
				}

				if (existingIndex >= 0) {
					devices[existingIndex] = updatedDevice;
				} else {
					devices.push(updatedDevice);
				}

				return [...devices];
			});
			return;
		}
	}

	if (typeof window !== 'undefined') {
		playbackApi.addEventHandler(handleSSEEvent);
		playbackApi.getEventStream();
	}

	return {
		subscribe,
		set,
		update,

		upsertFromStatus: (clientInfo: ClientInfo) => {
			update((devices) => {
				const deviceIndex = devices.findIndex((d) => d.device_id === clientInfo.device_id);
				const device: Device = {
					device_id: clientInfo.device_id,
					display_name: clientInfo.display_name,
					status: clientInfo.status as DeviceStatus,
					isConnected: true,
					last_activity: clientInfo.last_activity,
					healthy: clientInfo.status ? true : false
				};

				if (deviceIndex >= 0) {
					devices[deviceIndex] = device;
					return [...devices];
				} else {
					return [...devices, device];
				}
			});
		},

		setDisconnected: (deviceId: string) => {
			update((devices) => {
				const deviceIndex = devices.findIndex((d) => d.device_id === deviceId);
				if (deviceIndex >= 0) {
					const updatedDevices = [...devices];
					const existingDevice = updatedDevices[deviceIndex];
					if (existingDevice) {
						updatedDevices[deviceIndex] = {
							device_id: existingDevice.device_id,
							display_name: existingDevice.display_name,
							status: {
								...(existingDevice.status || {}),
								connected: false,
								healthy: false
							},
							isConnected: false,
							last_activity: existingDevice.last_activity,
							healthy: false
						};
					}
					return updatedDevices;
				}
				return devices;
			});
		},

		remove: (deviceId: string) => {
			update((devices) => {
				return devices.filter((d) => d.device_id !== deviceId);
			});
		},

		clear: () => {
			set([]);
		},

		getDevice: (deviceId: string): Device | undefined => {
			const devices = get(devicesStore);
			return devices.find((d) => d.device_id === deviceId);
		}
	};
};

export const devicesStore = createDevicesStore();

export const devices = derived(devicesStore, (map) => Array.from(map.values()));

const createActiveDeviceStore = () => {
	const { subscribe, set, update } = writable<ActiveDevice>({
		device_id: null,
		isConnected: false
	});

	return {
		subscribe,
		set,
		update,

		setActive: (deviceId: string | null) => {
			const currentDevices = get(devicesStore);
			const device = deviceId ? currentDevices.find((d) => d.device_id === deviceId) : undefined;

			const newActive: ActiveDevice = {
				device_id: deviceId,
				isConnected: device?.isConnected || false
			};
			if (device?.status) {
				newActive.status = device.status;
			}
			if (device) {
				newActive.device = device;
			}
			set(newActive);
		},

		updateStatus: (status: DeviceStatus) => {
			update((active) => {
				if (active.device_id) {
					const currentDevices = get(devicesStore);
					const device = currentDevices.find((d) => d.device_id === active.device_id);
					if (device) {
						return {
							...active,
							status: status || device.status,
							device: { ...device, status: status || device.status }
						};
					}
				}
				return active;
			});
		},

		setDisconnected: (deviceId: string) => {
			update((active) => {
				if (active.device_id === deviceId) {
					const result = {
						...active,
						isConnected: false
					};
					delete result.status;
					return result;
				}
				return active;
			});
		}
	};
};

export const activeDeviceStore = createActiveDeviceStore();

export function getActiveDevice(): ActiveDevice {
	return get(activeDeviceStore);
}
