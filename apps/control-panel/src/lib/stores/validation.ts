import { formatFileSize } from '../utils/format';

export interface ValidationResult {
	isValid: boolean;
	error?: string;
}

export class ValidationUtils {
	private static readonly MAX_FILE_SIZE = 500 * 1024 * 1024;
	private static readonly MAX_FILES = 50;
	private static readonly SUPPORTED_FORMATS = ['jpg', 'jpeg', 'png', 'gif', 'mp4'];

	static validateFileFormat(file: File): ValidationResult {
		const extension = file.name.split('.').pop()?.toLowerCase();
		const mimeType = file.type.toLowerCase();

		if (file.size > this.MAX_FILE_SIZE) {
			return {
				isValid: false,
				error: `File too large: ${file.name} (${formatFileSize(file.size)}). Maximum size: ${this.MAX_FILE_SIZE / (1024 * 1024)}MB`
			};
		}

		const isImage =
			['jpg', 'jpeg', 'png'].includes(extension || '') &&
			['image/jpeg', 'image/jpg', 'image/png'].includes(mimeType);
		const isGif = extension === 'gif' && mimeType === 'image/gif';
		const isVideo = extension === 'mp4' && mimeType === 'video/mp4';

		if (isImage || isGif || isVideo) {
			return { isValid: true };
		}

		return {
			isValid: false,
			error: `Unsupported file format: ${extension} (${mimeType}). Supported formats: ${this.SUPPORTED_FORMATS.join(', ').toUpperCase()}`
		};
	}

	static validateUploadLimits(files: File[]): ValidationResult {
		if (files.length > this.MAX_FILES) {
			return {
				isValid: false,
				error: `Too many files: ${files.length}. Maximum allowed: ${this.MAX_FILES} files`
			};
		}

		for (const file of files) {
			const validation = this.validateFileFormat(file);
			if (!validation.isValid) {
				return validation;
			}
		}

		return { isValid: true };
	}
}
