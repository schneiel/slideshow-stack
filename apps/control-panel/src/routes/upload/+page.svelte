<script lang="ts">
	import { CloudUpload, Image, Video, RotateCw, FolderOpen } from '@lucide/svelte';
	import { mediaApi } from '$lib/api/media';
	import { ValidationUtils } from '$lib/stores/validation';
	import { formatFileSize } from '$lib/utils/format';
	import { showSuccess } from '$lib/stores/toast';
	import { showErrorFromApi } from '$lib/utils/errors';

	let uploadLoading = $state(false);
	let dragOver = $state(false);
	let selectedFiles = $state<File[]>([]);
	let fileInput: HTMLInputElement;

	async function uploadFiles(files: File[]): Promise<void> {
		if (!files || files.length === 0) return;

		const validation = ValidationUtils.validateUploadLimits(files);
		if (!validation.isValid) {
			showErrorFromApi({
				type: 'validation',
				message: validation.error || 'File validation failed',
				retryable: false
			});
			return;
		}

		try {
			uploadLoading = true;

			const response = await mediaApi.uploadFiles(files);

			if (!response) {
				showErrorFromApi({
					type: 'unknown',
					message: 'Upload failed: No response from server',
					retryable: true
				});
				return;
			}

			const { uploaded_files, upload_errors } = response;

			const uploaded = uploaded_files || [];
			const errors = upload_errors || [];

			if (uploaded.length > 0) {
				const fileCount = uploaded.length;
				showSuccess(`Successfully uploaded ${fileCount} file${fileCount !== 1 ? 's' : ''}`);
			}

			if (errors.length > 0) {
				const errorCount = errors.length;
				showErrorFromApi({
					type: 'validation',
					message: `${errorCount} file${errorCount !== 1 ? 's' : ''} failed to upload: ${errors.join(', ')}`,
					retryable: false
				});
			}

			if (uploaded.length === 0 && errors.length === 0) {
				showErrorFromApi({
					type: 'unknown',
					message: 'No files were uploaded',
					retryable: true
				});
			}

			if (uploaded.length > 0 || errors.length > 0) {
				selectedFiles = [];
				if (fileInput) fileInput.value = '';
			}
		} catch (e) {
			showErrorFromApi(e);
		} finally {
			uploadLoading = false;
		}
	}

	function handleDrop(event: DragEvent) {
		event.preventDefault();
		dragOver = false;

		const files = Array.from(event.dataTransfer?.files || []);
		selectedFiles = files;

		if (files.length === 1) {
			uploadFiles(files);
		}
	}

	function handleDragOver(event: DragEvent) {
		event.preventDefault();
		dragOver = true;
	}

	function handleDragLeave(event: DragEvent) {
		event.preventDefault();
		dragOver = false;
	}

	function handleChange(event: Event) {
		const target = event.target as HTMLInputElement;
		const files = Array.from(target.files || []);
		selectedFiles = files;

		if (files.length === 1) {
			uploadFiles(files);
		}
	}

	async function uploadMultipleFiles(): Promise<void> {
		await uploadFiles(selectedFiles);
	}

	function getTotalFileSize(): number {
		return selectedFiles.reduce((total: number, file: File) => total + file.size, 0);
	}
</script>

<svelte:head>
	<title>Upload Media Files - Slideshow Stack</title>
</svelte:head>

<div class="upload-page">
	<!-- Header -->
	<div class="page-header">
		<div class="header-content">
			<div class="title-section">
				<h1>Upload Media Files</h1>
				<p>Add images and videos to your media library</p>
			</div>
			<div class="header-stats">
				<div class="stat-card">
					<div class="stat-value">500MB</div>
					<div class="stat-label">Max File Size</div>
				</div>
			</div>
		</div>
	</div>

	<!-- Upload Section -->
	<div class="upload-section">
		<div
			class="upload-area"
			class:drag-over={dragOver}
			role="button"
			tabindex="0"
			aria-label="Drop files here or click to upload"
			ondrop={handleDrop}
			ondragover={handleDragOver}
			ondragleave={handleDragLeave}
			onkeydown={(e: KeyboardEvent) => {
				if (e.key === 'Enter' || e.key === ' ') {
					fileInput?.click();
				}
			}}
			onclick={() => fileInput?.click()}
		>
			<input
				type="file"
				id="file-upload"
				bind:this={fileInput}
				accept="image/jpeg,image/jpg,image/png,image/gif,video/mp4"
				multiple={true}
				onchange={handleChange}
				disabled={uploadLoading}
			/>

			<div class="upload-content">
				{#if selectedFiles.length > 0}
					<div class="selected-files">
						<FolderOpen size={48} class="upload-icon" />
						<h3>Selected Files ({selectedFiles.length})</h3>
						<div class="files-list">
							{#each selectedFiles as file (file.name)}
								<div class="file-item">
									<div class="file-info">
										<strong>{file.name}</strong>
										<span class="file-type">
											{file.type || 'Unknown'}
										</span>
									</div>
									<span class="file-size">{formatFileSize(file.size)}</span>
								</div>
							{/each}
						</div>
						<div class="total-size">
							<span>Total: {formatFileSize(getTotalFileSize())}</span>
						</div>

						{#if selectedFiles.length > 1}
							<button class="btn-upload" onclick={uploadMultipleFiles} disabled={uploadLoading}>
								<CloudUpload size={20} />
								<span>Upload All Files</span>
							</button>
						{/if}
					</div>
				{:else}
					<div class="upload-prompt">
						<div class="upload-icon-wrapper">
							<CloudUpload size={64} class="upload-icon" />
						</div>
						<h3>Upload Media Files</h3>
						<p>Drag & drop images or videos here, or click to browse</p>
						<div class="supported-formats">
							<div class="format-tag">
								<Image size={16} />
								<span>JPEG, PNG, GIF</span>
							</div>
							<div class="format-tag">
								<Video size={16} />
								<span>MP4</span>
							</div>
						</div>
					</div>
				{/if}
			</div>

			{#if uploadLoading}
				<div class="upload-overlay">
					<RotateCw size={48} class="loading-icon" />
					<span>Uploading...</span>
				</div>
			{/if}
		</div>
	</div>

	<!-- Supported Formats -->
	<div class="formats-section">
		<h2>Supported Formats</h2>
		<div class="formats-grid">
			<div class="format-card">
				<div class="format-icon">
					<Image size={32} />
				</div>
				<h4>Images</h4>
				<p>JPEG, PNG, GIF</p>
				<span class="format-detail">Perfect for photos and graphics</span>
			</div>
			<div class="format-card">
				<div class="format-icon">
					<Video size={32} />
				</div>
				<h4>Videos</h4>
				<p>MP4</p>
				<span class="format-detail">High-quality video support</span>
			</div>
		</div>
	</div>
</div>

<style>
	.upload-page {
		padding: var(--space-6);
		max-width: 1600px;
		margin: 0 auto;
		min-height: 100vh;
		background: linear-gradient(135deg, #f8fafc 0%, #f1f5f9 100%);
	}

	.page-header {
		margin-bottom: var(--space-8);
		background: white;
		border-radius: var(--radius-xl);
		padding: var(--space-6);
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
	}

	.header-content {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		gap: var(--space-4);
	}

	.title-section h1 {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		margin: 0 0 var(--space-2) 0;
		font-size: var(--font-size-3xl);
		font-weight: var(--font-weight-bold);
		color: var(--color-text);
	}

	.title-section p {
		margin: 0;
		color: var(--color-text-muted);
		font-size: var(--font-size-lg);
	}

	.header-stats {
		display: flex;
		gap: var(--space-4);
	}

	.stat-card {
		text-align: center;
		padding: var(--space-4);
		background: var(--color-bg-secondary);
		border-radius: var(--radius-lg);
		min-width: 120px;
	}

	.stat-value {
		display: block;
		font-size: var(--font-size-2xl);
		font-weight: var(--font-weight-bold);
		color: var(--color-primary);
	}

	.stat-label {
		display: block;
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
		margin-top: var(--space-1);
	}

	.upload-section {
		background: white;
		border-radius: var(--radius-xl);
		padding: var(--space-6);
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
		margin-bottom: var(--space-6);
	}

	.upload-area {
		border: 3px dashed var(--color-border);
		border-radius: var(--radius-xl);
		padding: var(--space-8);
		text-align: center;
		transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
		background: var(--color-bg-secondary);
		position: relative;
		min-height: 350px;
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
	}

	.upload-area:hover,
	.upload-area.drag-over {
		border-color: var(--color-primary);
		background: rgba(141, 198, 63, 0.05);
		transform: translateY(-2px);
		box-shadow: 0 8px 25px rgba(0, 0, 0, 0.1);
	}

	.upload-content {
		pointer-events: none;
		max-width: 600px;
		width: 100%;
	}

	.upload-icon-wrapper {
		display: flex;
		justify-content: center;
		margin-bottom: var(--space-4);
	}

	.upload-prompt h3 {
		font-size: var(--font-size-2xl);
		font-weight: var(--font-weight-semibold);
		margin-bottom: var(--space-2);
		color: var(--color-text);
	}

	.upload-prompt p {
		color: var(--color-text-muted);
		margin-bottom: var(--space-4);
		font-size: var(--font-size-lg);
	}

	.supported-formats {
		display: flex;
		gap: var(--space-3);
		justify-content: center;
		margin-top: var(--space-4);
	}

	.format-tag {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		padding: var(--space-2) var(--space-4);
		background: white;
		border-radius: var(--radius-full);
		font-size: var(--font-size-sm);
		color: var(--color-text);
		border: 2px solid var(--color-primary);
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
	}

	.selected-files {
		background: white;
		padding: var(--space-6);
		border-radius: var(--radius-lg);
		border: 2px solid var(--color-border);
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
	}

	.selected-files h3 {
		font-size: var(--font-size-xl);
		font-weight: var(--font-weight-semibold);
		margin-bottom: var(--space-4);
		color: var(--color-text);
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.files-list {
		max-height: 250px;
		overflow-y: auto;
		margin-bottom: var(--space-4);
	}

	.file-item {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: var(--space-3);
		background: var(--color-bg-secondary);
		border-radius: var(--radius-md);
		margin-bottom: var(--space-2);
		border-left: 3px solid var(--color-primary);
		transition: var(--transition-base);
	}

	.file-item:hover {
		transform: translateX(4px);
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
	}

	.file-info {
		flex: 1;
		min-width: 0;
	}

	.file-info strong {
		color: var(--color-text);
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-semibold);
		display: block;
		margin-bottom: var(--space-1);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.file-type {
		color: var(--color-text-muted);
		font-size: var(--font-size-xs);
	}

	.file-size {
		color: var(--color-primary);
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-medium);
	}

	.total-size {
		text-align: center;
		font-weight: var(--font-weight-bold);
		color: var(--color-primary);
		margin: var(--space-4) 0;
		padding: var(--space-3);
		background: var(--color-bg-secondary);
		border-radius: var(--radius-md);
		font-size: var(--font-size-lg);
	}

	.btn-upload {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		padding: var(--space-3) var(--space-6);
		background: var(--color-primary);
		color: white;
		border: none;
		border-radius: var(--radius-md);
		font-size: var(--font-size-base);
		font-weight: var(--font-weight-semibold);
		cursor: pointer;
		transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
		margin-top: var(--space-4);
		pointer-events: all;
	}

	.btn-upload:hover:not(:disabled) {
		background: var(--color-primary-hover);
		transform: translateY(-2px);
		box-shadow: 0 4px 15px rgba(141, 198, 63, 0.3);
	}

	.btn-upload:disabled {
		background: var(--color-text-muted);
		cursor: not-allowed;
		transform: none;
	}

	.upload-overlay {
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background: rgba(255, 255, 255, 0.98);
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: var(--space-4);
		backdrop-filter: blur(3px);
		border-radius: var(--radius-xl);
	}

	.upload-overlay span {
		color: var(--color-text);
		font-size: var(--font-size-lg);
		font-weight: var(--font-weight-semibold);
	}

	.formats-section {
		background: white;
		border-radius: var(--radius-xl);
		padding: var(--space-6);
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
		margin-bottom: var(--space-6);
	}

	.formats-section h2 {
		font-size: var(--font-size-xl);
		font-weight: var(--font-weight-semibold);
		margin-bottom: var(--space-6);
		color: var(--color-text);
		text-align: center;
	}

	.formats-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
		gap: var(--space-6);
	}

	.format-card {
		background: var(--color-bg-secondary);
		padding: var(--space-6);
		border-radius: var(--radius-lg);
		border: 2px solid transparent;
		text-align: center;
		transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
	}

	.format-card:hover {
		border-color: var(--color-primary);
		transform: translateY(-2px);
		box-shadow: 0 8px 25px rgba(0, 0, 0, 0.1);
	}

	.format-icon {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 64px;
		height: 64px;
		background: rgba(141, 198, 63, 0.1);
		color: var(--color-primary);
		border-radius: var(--radius-lg);
		margin: 0 auto var(--space-4);
	}

	.format-card h4 {
		font-size: var(--font-size-lg);
		font-weight: var(--font-weight-semibold);
		margin-bottom: var(--space-2);
		color: var(--color-text);
	}

	.format-card p {
		color: var(--color-text-muted);
		font-size: var(--font-size-sm);
		margin-bottom: var(--space-2);
	}

	.format-detail {
		display: block;
		color: var(--color-text-muted);
		font-size: var(--font-size-xs);
		font-style: italic;
	}

	#file-upload {
		display: none;
	}

	@media (max-width: 768px) {
		.upload-page {
			padding: var(--space-4);
		}

		.header-content {
			flex-direction: column;
			gap: var(--space-4);
			align-items: stretch;
		}

		.header-stats {
			justify-content: space-around;
		}

		.formats-grid {
			grid-template-columns: 1fr;
		}

		.upload-area {
			min-height: 280px;
			padding: var(--space-6);
		}
	}

	@media (max-width: 480px) {
		.upload-page {
			padding: var(--space-3);
		}

		.page-header {
			padding: var(--space-4);
		}

		.title-section h1 {
			font-size: var(--font-size-2xl);
		}

		.title-section p {
			font-size: var(--font-size-sm);
		}

		.upload-area {
			min-height: 220px;
			padding: var(--space-4);
		}

		.upload-prompt h3 {
			font-size: var(--font-size-lg);
		}

		.upload-prompt p {
			font-size: var(--font-size-sm);
		}

		.supported-formats {
			flex-direction: column;
			gap: var(--space-2);
		}

		.format-tag {
			font-size: var(--font-size-xs);
			padding: var(--space-2) var(--space-3);
		}

		.selected-files {
			padding: var(--space-4);
		}

		.selected-files h3 {
			font-size: var(--font-size-base);
		}

		.btn-upload {
			width: 100%;
			justify-content: center;
			padding: var(--space-4);
			min-height: 48px;
		}

		.formats-section {
			padding: var(--space-4);
		}

		.formats-grid {
			gap: var(--space-4);
		}

		.format-card {
			padding: var(--space-4);
		}
	}

	@media (max-width: 390px) {
		.upload-page {
			padding: var(--space-2);
		}

		.page-header {
			padding: var(--space-3);
		}

		.title-section h1 {
			font-size: var(--font-size-xl);
		}

		.stat-card {
			min-width: 80px;
			padding: var(--space-3);
		}

		.stat-value {
			font-size: var(--font-size-lg);
		}
	}
</style>
