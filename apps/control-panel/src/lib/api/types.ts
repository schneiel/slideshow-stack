export interface MediaMetadata {
	filename: string;
	path: string;
	size: number;
	mod_time: string;
	media_type: 'image' | 'video';
}

export interface UploadResponse {
	uploaded_files: string[];
	upload_errors: string[];
}

export interface DeleteMediaResponse {
	message: string;
	filename: string;
}

export interface SlideshowSummary {
	id: string;
	name: string;
	description?: string;
	media_ids: string[];
	interval_seconds: number;
	loop_enabled: boolean;
	shuffle: boolean;
	auto_start: boolean;
	created_at: string;
	updated_at: string;
}

export type SlideshowResponse = SlideshowSummary;

export interface SlideshowRequest {
	name: string;
	description?: string;
	media_ids: string[];
	interval_seconds: number;
	loop_enabled: boolean;
	shuffle: boolean;
	auto_start: boolean;
}

export interface UpdateSlideshowRequest {
	name?: string;
	description?: string;
	media_ids?: string[];
	interval_seconds?: number;
	loop_enabled?: boolean;
	shuffle?: boolean;
	auto_start?: boolean;
}
