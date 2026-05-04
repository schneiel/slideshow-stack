<script lang="ts">
	import { Monitor, CloudUpload, Play, File, Trash2 } from '@lucide/svelte';
	import { onMount } from 'svelte';
	import { mediaApi } from '$lib/api/media';
	import { formatFileSize } from '$lib/utils/format';
	import { type MediaMetadata } from '$lib/api/types';
	import { showSuccess } from '$lib/stores/toast';
	import { startSingleImage } from '$lib/stores/playback';
	import { getMediaUrl } from '$lib/utils/mediaUrls';
	import DeviceSelectionModal from '$lib/components/DeviceSelectionModal.svelte';
	import { showErrorFromApi } from '$lib/utils/errors';

	let mediaFiles = $state<MediaMetadata[]>([]);
	let loading = $state(true);

	let showDeleteConfirm = $state(false);
	let deleteConfirmCallback = $state<(() => void) | null>(null);
	let deleteConfirmFile = $state<MediaMetadata | null>(null);

	let showDeviceModal = $state(false);
	let pendingImage = $state<MediaMetadata | null>(null);

	function confirmDelete(callback: () => void, file: MediaMetadata): void {
		deleteConfirmCallback = callback;
		deleteConfirmFile = file;
		showDeleteConfirm = true;
	}

	function cancelDelete(): void {
		showDeleteConfirm = false;
		deleteConfirmCallback = null;
		deleteConfirmFile = null;
	}

	function confirmDeleteAction(): void {
		if (deleteConfirmCallback) {
			deleteConfirmCallback();
		}
		cancelDelete();
	}

	onMount(async () => {
		await loadMedia();
	});

	async function loadMedia(): Promise<void> {
		try {
			loading = true;
			const files = await mediaApi.getMedia();
			mediaFiles = Array.isArray(files) ? files : [];
		} catch (e) {
			mediaFiles = [];
			showErrorFromApi(e);
		} finally {
			loading = false;
		}
	}

	async function deleteMedia(file: MediaMetadata): Promise<void> {
		confirmDelete(async () => {
			try {
				const response = await mediaApi.deleteMedia(file.filename);
				showSuccess(response.message);
				await loadMedia();
			} catch (e) {
				showErrorFromApi(e);
			}
		}, file);
	}

	function isImage(mediaType: string): boolean {
		return mediaType === 'image';
	}

	function getMediaIcon(mediaType: string): string {
		if (mediaType === 'video') return 'VIDEO';
		if (mediaType === 'image') return 'IMAGE';
		return 'FILE';
	}

	function openPlayModal(image: MediaMetadata): void {
		pendingImage = image;
		showDeviceModal = true;
	}

	async function handleDeviceConfirm(targetClientIds: string[]): Promise<void> {
		if (!pendingImage) return;
		await startSingleImage(pendingImage.filename, targetClientIds);
		showDeviceModal = false;
		pendingImage = null;
	}

	function handleDeviceCancel(): void {
		showDeviceModal = false;
		pendingImage = null;
	}
</script>

<svelte:head>
	<title>Media Library - Slideshow Stack</title>
</svelte:head>

<div class="media-library">
	<!-- Header -->
	<div class="page-header">
		<div class="header-content">
			<div class="title-section">
				<h1>Media Library</h1>
				<p>Manage and control media playback for presentations</p>
			</div>
			<div class="header-stats">
				<div class="stat-card">
					<div class="stat-value">{mediaFiles?.length ?? 0}</div>
					<div class="stat-label">Total Files</div>
				</div>
				<div class="stat-card">
					<div class="stat-value">
						{(mediaFiles ?? []).filter((f) => f.media_type === 'image').length}
					</div>
					<div class="stat-label">Images</div>
				</div>
				<div class="stat-card">
					<div class="stat-value">
						{(mediaFiles ?? []).filter((f) => f.media_type === 'video').length}
					</div>
					<div class="stat-label">Videos</div>
				</div>
			</div>
		</div>
	</div>

	<!-- Media Grid Section -->
	<div class="media-section">
		{#if loading}
			<div class="loading-state">
				<div class="loading-spinner"></div>
				<p>Loading media files...</p>
			</div>
		{:else if (mediaFiles?.length ?? 0) === 0}
			<div class="empty-state">
				<Monitor size={64} />
				<h3>No media files found</h3>
				<p>Upload some files to get started!</p>
				<a href="/upload" class="btn-primary" rel="external">
					<CloudUpload size={20} />
					<span>Upload Files</span>
				</a>
			</div>
		{:else}
			<div class="media-grid">
				{#each mediaFiles ?? [] as file (file.filename)}
					<div class="media-card">
						<div class="card-header">
							<div class="media-avatar">
								{#if isImage(file.media_type)}
									<img src={getMediaUrl(file.filename)} alt={file.filename} loading="lazy" />
								{:else if file.media_type === 'video'}
									<Play size={24} class="video-icon" />
								{:else}
									<File size={24} class="file-icon" />
								{/if}
							</div>
							<div class="media-info">
								<h3>{file.filename}</h3>
								<div class="media-meta">
									<span class="media-type">
										{getMediaIcon(file.media_type)} - {file.media_type}
									</span>
									<span class="media-size">{formatFileSize(file.size)}</span>
								</div>
							</div>
						</div>

						<div class="media-details">
							<div class="detail-item">
								<span class="detail-label">Type:</span>
								<span class="detail-value">
									{file.filename.split('.').pop()?.toUpperCase() || 'Unknown'}
								</span>
							</div>
							<div class="detail-item">
								<span class="detail-label">Size:</span>
								<span class="detail-value">{formatFileSize(file.size)}</span>
							</div>
						</div>

						<div class="card-actions">
							{#if isImage(file.media_type) || file.media_type === 'video'}
								<button
									onclick={() => openPlayModal(file)}
									class="btn-play"
									type="button"
									aria-label={`Play ${file.filename}`}
								>
									<Play size={16} />
									<span>Play</span>
								</button>
							{/if}
							<a
								href={getMediaUrl(file.filename)}
								download={file.filename}
								class="btn-download"
								rel="external"
							>
								<File size={16} />
								<span>Download</span>
							</a>
							<button
								onclick={() => deleteMedia(file)}
								class="btn-delete"
								type="button"
								aria-label={`Delete ${file.filename}`}
							>
								<Trash2 size={16} />
								<span>Delete</span>
							</button>
						</div>
					</div>
				{/each}
			</div>
		{/if}
	</div>
</div>

<!-- Delete Confirmation Modal -->
{#if showDeleteConfirm && deleteConfirmFile}
	<div
		class="modal-overlay"
		role="dialog"
		aria-modal="true"
		onclick={cancelDelete}
		onkeydown={(e: KeyboardEvent) => {
			if (e.key === 'Escape') cancelDelete();
		}}
		tabindex="-1"
	>
		<div class="modal-content" role="presentation" onclick={(e: MouseEvent) => e.stopPropagation()}>
			<h3>Confirm Deletion</h3>
			<p>Are you sure you want to delete "{deleteConfirmFile.filename}"?</p>
			<p class="modal-warning">This action cannot be undone.</p>
			<div class="modal-actions">
				<button class="btn-cancel" onclick={cancelDelete}>Cancel</button>
				<button class="btn-delete" onclick={confirmDeleteAction}>Delete</button>
			</div>
		</div>
	</div>
{/if}

<DeviceSelectionModal
	show={showDeviceModal}
	description={pendingImage ? `Play: ${pendingImage.filename}` : ''}
	onConfirm={handleDeviceConfirm}
	onCancel={handleDeviceCancel}
/>

<style>
	.media-library {
		padding: var(--space-6);
		max-width: 1600px;
		margin: 0 auto;
		min-height: 100vh;
		background: linear-gradient(135deg, #f8fafc 0%, #f1f5f9 100%);
	}

	/* Header */
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

	/* Media Section */
	.media-section {
		background: white;
		border-radius: var(--radius-xl);
		padding: var(--space-6);
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
	}

	.loading-state,
	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-4);
		padding: var(--space-12);
		text-align: center;
		color: var(--color-text-muted);
	}

	.empty-state h3 {
		margin: 0;
		font-size: var(--font-size-xl);
		font-weight: var(--font-weight-semibold);
		color: var(--color-text);
	}

	.loading-spinner {
		width: 32px;
		height: 32px;
		border: 3px solid var(--color-bg-secondary);
		border-top: 3px solid var(--color-primary);
		border-radius: 50%;
		animation: spin 1s linear infinite;
	}

	/* Media Grid */
	.media-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
		gap: var(--space-6);
	}

	.media-card {
		background: var(--color-bg-secondary);
		border-radius: var(--radius-lg);
		padding: var(--space-5);
		transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
		border: 2px solid transparent;
		overflow: hidden;
	}

	.media-card:hover {
		transform: translateY(-2px);
		box-shadow: 0 8px 25px rgba(0, 0, 0, 0.15);
		border-color: var(--color-primary);
	}

	.card-header {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		margin-bottom: var(--space-4);
	}

	.media-avatar {
		width: 48px;
		height: 48px;
		background: var(--color-primary);
		color: white;
		border-radius: var(--radius-lg);
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
		overflow: hidden;
	}

	.media-avatar img {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}

	.media-info {
		flex: 1;
		min-width: 0;
	}

	.media-info h3 {
		margin: 0 0 var(--space-1) 0;
		font-size: var(--font-size-base);
		font-weight: var(--font-weight-semibold);
		color: var(--color-text);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.media-meta {
		display: flex;
		gap: var(--space-2);
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
	}

	.media-details {
		margin-bottom: var(--space-4);
		padding: var(--space-3);
		background: white;
		border-radius: var(--radius-md);
	}

	.detail-item {
		display: flex;
		justify-content: space-between;
		font-size: var(--font-size-sm);
		margin-bottom: var(--space-2);
	}

	.detail-item:last-child {
		margin-bottom: 0;
	}

	.detail-label {
		color: var(--color-text-muted);
	}

	.detail-value {
		font-weight: var(--font-weight-medium);
		color: var(--color-text);
	}

	.card-actions {
		display: flex;
		gap: var(--space-3);
	}

	.btn-play,
	.btn-download,
	.btn-delete {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: var(--space-2);
		padding: var(--space-3);
		border: none;
		border-radius: var(--radius-md);
		font-weight: var(--font-weight-medium);
		cursor: pointer;
		transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
		font-size: var(--font-size-sm);
		text-decoration: none;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		max-width: 100%;
	}

	.btn-play {
		background: var(--color-primary);
		color: white;
	}

	.btn-play:hover:not(:disabled) {
		background: var(--color-primary-hover);
		transform: translateY(-1px);
		box-shadow: 0 4px 15px rgba(141, 198, 63, 0.3);
	}

	.btn-download {
		background: var(--color-secondary);
		color: white;
	}

	.btn-download:hover:not(:disabled) {
		background: var(--color-secondary-hover);
		transform: translateY(-1px);
	}

	.btn-delete {
		background: var(--color-danger);
		color: white;
	}

	.btn-delete:hover:not(:disabled) {
		background: var(--color-danger-dark);
		transform: translateY(-1px);
		box-shadow: 0 4px 15px rgba(231, 76, 60, 0.3);
	}

	.btn-primary {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		padding: var(--space-3) var(--space-6);
		background: var(--color-primary);
		color: white;
		text-decoration: none;
		border-radius: var(--radius-md);
		font-weight: var(--font-weight-medium);
		transition: var(--transition-base);
	}

	.btn-primary:hover {
		background: var(--color-primary-hover);
		transform: translateY(-1px);
	}

	/* Modal */
	.modal-overlay {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background: rgba(0, 0, 0, 0.8);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
		animation: fadeIn 0.2s ease;
	}

	.modal-content {
		background: white;
		border-radius: var(--radius-xl);
		padding: var(--space-6);
		max-width: 600px;
		width: 90%;
		max-height: 90vh;
		overflow-y: auto;
		animation: slideUp 0.3s ease;
	}

	.modal-content h3 {
		margin: 0 0 var(--space-4) 0;
		font-size: var(--font-size-xl);
		font-weight: var(--font-weight-semibold);
		color: var(--color-text);
	}

	.modal-content p {
		margin: 0 0 var(--space-3) 0;
		color: var(--color-text);
		line-height: var(--line-height-relaxed);
	}

	.modal-warning {
		color: var(--color-danger);
		font-weight: var(--font-weight-semibold);
		margin-top: var(--space-4);
	}

	.modal-actions {
		display: flex;
		gap: var(--space-3);
		justify-content: flex-end;
		margin-top: var(--space-6);
	}

	.btn-cancel,
	.btn-delete {
		padding: var(--space-3) var(--space-6);
		border: none;
		border-radius: var(--radius-md);
		font-weight: var(--font-weight-medium);
		cursor: pointer;
		transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
		font-size: var(--font-size-base);
	}

	.btn-cancel {
		background: var(--color-bg-secondary);
		color: var(--color-text);
	}

	.btn-cancel:hover:not(:disabled) {
		background: var(--color-border);
	}

	.btn-delete {
		background: var(--color-danger);
		color: white;
	}

	.btn-delete:hover:not(:disabled) {
		background: var(--color-danger-dark);
		transform: translateY(-1px);
		box-shadow: 0 4px 15px rgba(231, 76, 60, 0.3);
	}

	/* Animations */
	@keyframes fadeIn {
		from {
			opacity: 0;
		}
		to {
			opacity: 1;
		}
	}

	@keyframes slideUp {
		from {
			opacity: 0;
			transform: translateY(20px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}

	@keyframes spin {
		0% {
			transform: rotate(0deg);
		}
		100% {
			transform: rotate(360deg);
		}
	}

	/* Responsive - Tablet */
	@media (max-width: 768px) {
		.media-library {
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

		.media-grid {
			grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
			gap: var(--space-4);
		}

		.modal-content {
			width: 95%;
			padding: var(--space-4);
		}

		.modal-actions {
			flex-direction: column-reverse;
		}

		.btn-cancel,
		.btn-delete {
			width: 100%;
		}
	}

	/* Responsive - Mobile Large (480px) */
	@media (max-width: 480px) {
		.media-library {
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

		.stat-card {
			min-width: 80px;
			padding: var(--space-3);
		}

		.stat-value {
			font-size: var(--font-size-xl);
		}

		.media-section {
			padding: var(--space-4);
		}

		.media-grid {
			grid-template-columns: 1fr;
			gap: var(--space-3);
		}

		.media-card {
			padding: var(--space-4);
		}

		.media-details {
			padding: var(--space-2);
		}

		.card-actions {
			flex-direction: column;
			gap: var(--space-2);
		}

		.btn-play,
		.btn-download,
		.btn-delete {
			padding: var(--space-3) var(--space-4);
			min-height: 44px;
			white-space: normal;
			text-align: center;
		}

		.btn-play span,
		.btn-download span,
		.btn-delete span {
			display: inline;
		}

		.media-info h3 {
			font-size: var(--font-size-sm);
		}
	}

	/* Responsive - Mobile Small (390px - iPhone) */
	@media (max-width: 390px) {
		.media-library {
			padding: var(--space-2);
		}

		.page-header {
			padding: var(--space-3);
		}

		.title-section h1 {
			font-size: var(--font-size-xl);
		}

		.header-stats {
			flex-wrap: wrap;
			gap: var(--space-2);
		}

		.stat-card {
			flex: 1;
			min-width: 70px;
			padding: var(--space-2);
		}

		.stat-value {
			font-size: var(--font-size-lg);
		}

		.stat-label {
			font-size: var(--font-size-xs);
		}

		.media-section {
			padding: var(--space-3);
		}

		.empty-state {
			padding: var(--space-8) var(--space-4);
		}

		.empty-state h3 {
			font-size: var(--font-size-lg);
		}
	}

	/* Touch-friendly improvements */
	@media (hover: none) and (pointer: coarse) {
		.media-card:hover {
			transform: none;
		}

		button:active:not(:disabled) {
			transform: scale(0.98);
		}
	}
</style>
