<script lang="ts">
	import { onMount } from 'svelte';
	import { SvelteSet } from 'svelte/reactivity';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { mediaApi } from '$lib/api/media';
	import { slideshowApi } from '$lib/api/slideshow';
	import { formatFileSize } from '$lib/utils/format';
	import { getMediaUrl } from '$lib/utils/mediaUrls';
	import type { MediaMetadata, SlideshowSummary, SlideshowResponse } from '$lib/api/types';
	import { showSuccess, showWarning } from '$lib/stores/toast';
	import { showErrorFromApi } from '$lib/utils/errors';

	import { startSavedSlideshow, startSlideshowWithConfig } from '$lib/stores/playback';

	import DeviceSelectionModal from '$lib/components/DeviceSelectionModal.svelte';
	import {
		FolderOpen,
		Edit,
		Trash2,
		Shuffle,
		Film,
		Inbox,
		CheckCircle,
		Play,
		CloudUpload
	} from '@lucide/svelte';

	let images = $state<MediaMetadata[]>([]);
	let loading = $state(true);

	// Simple slideshow state
	let selectedImages = $state<string[]>([]);
	let interval = $state(5);
	let slideshowName = $state('');
	let shuffle = $state(false);
	let savedSlideshows = $state<SlideshowSummary[]>([]);
	let currentWorkflow = $state<'create' | 'saved' | 'active'>('saved');
	let editingSlideshow = $state<SlideshowResponse | null>(null);
	let showDeleteConfirm = $state(false);
	let deleteConfirmCallback = $state<(() => void) | null>(null);

	let showDeviceModal = $state(false);
	let pendingPlayAction = $state<null | {
		type: 'saved' | 'unsaved';
		slideshow?: SlideshowSummary;
	}>(null);

	function confirmDelete(callback: () => void): void {
		deleteConfirmCallback = callback;
		showDeleteConfirm = true;
	}

	function cancelDelete(): void {
		showDeleteConfirm = false;
		deleteConfirmCallback = null;
	}

	function confirmDeleteAction(): void {
		if (deleteConfirmCallback) {
			deleteConfirmCallback();
		}
		cancelDelete();
	}

	let draggedImage: string | null = $state(null);
	let draggedOverImage: string | null = $state(null);

	let selectedInPool = $state<Set<string>>(new Set());

	onMount(() => {
		const loadInitialData = async () => {
			await loadImages();
			await loadSavedSlideshows();

			const editId = $page.url.searchParams.get('edit');

			if (editId) {
				try {
					const slideshowToEdit = await slideshowApi.getSlideshow(editId);

					if (!slideshowToEdit) {
						throw new Error('Slideshow not found');
					}

					await startEditSlideshow(slideshowToEdit);
				} catch {
					// eslint-disable-next-line svelte/no-navigation-without-resolve
					await goto('/slideshow', { replaceState: true });

					showErrorFromApi(
						`The slideshow you're trying to edit no longer exists or has been moved.`
					);
				}
			}
		};

		loadInitialData();
	});

	async function loadImages(): Promise<void> {
		try {
			loading = true;
			const allMedia = await mediaApi.getMedia();
			images = allMedia.filter((m) => m.media_type === 'image');
		} catch {
			images = [];
			showErrorFromApi('Backend not reachable. Please make sure the API server is running.');
		} finally {
			loading = false;
		}
	}

	async function loadSavedSlideshows(): Promise<void> {
		try {
			const slideshows = await slideshowApi.getSlideshows();
			savedSlideshows = slideshows;
		} catch {
			savedSlideshows = [];
		}
	}

	function openDeviceModal(actionType: 'saved' | 'unsaved', slideshow?: SlideshowSummary) {
		pendingPlayAction = { type: actionType, ...(slideshow && { slideshow }) };
		showDeviceModal = true;
	}

	function closeDeviceModal() {
		showDeviceModal = false;
		pendingPlayAction = null;
	}

	async function handleDeviceConfirm(targetClientIds: string[]) {
		if (!pendingPlayAction) return;

		if (pendingPlayAction.type === 'saved' && pendingPlayAction.slideshow) {
			await startSavedSlideshow(pendingPlayAction.slideshow.id, targetClientIds);
		} else if (pendingPlayAction.type === 'unsaved') {
			if (selectedImages.length === 0) {
				showWarning('Please select at least one image for the slideshow');
				return;
			}
			await startSlideshowWithConfig(
				{
					name: slideshowName || 'Unsaved Slideshow',
					media_ids: selectedImages,
					interval_seconds: interval,
					shuffle: shuffle,
					loop_enabled: true,
					scaling_mode: 'fit'
				},
				targetClientIds
			);
		}

		closeDeviceModal();
	}

	function getModalDescription(): string {
		if (pendingPlayAction?.type === 'saved' && pendingPlayAction.slideshow) {
			return `Play: ${pendingPlayAction.slideshow.name}`;
		}
		return `Play: ${slideshowName || 'Unsaved Slideshow'}`;
	}

	function toggleImageSelection(imageId: string): void {
		const image = images.find((img) => img.filename === imageId);
		if (!image) return;

		if (selectedImages.includes(imageId)) {
			selectedImages = selectedImages.filter((id) => id !== imageId);
		} else {
			selectedImages = [...selectedImages, imageId];
		}
	}

	async function saveSlideshow(): Promise<void> {
		if (selectedImages.length === 0) {
			showWarning('Please select at least one image for the slideshow');
			return;
		}

		if (!slideshowName.trim()) {
			showWarning('Please enter a name for the slideshow');
			return;
		}

		try {
			const response = await slideshowApi.createSlideshow({
				name: slideshowName,
				media_ids: [...selectedImages],
				interval_seconds: interval,
				loop_enabled: true,
				shuffle: shuffle,
				auto_start: false
			});

			showSuccess(`Slideshow "${response.name}" saved successfully!`);

			await loadSavedSlideshows();
			slideshowName = '';
			selectedImages = [];
		} catch {
			showErrorFromApi('Error saving slideshow. Please try again.');
		}
	}

	async function deleteSlideshow(slideshowId: string): Promise<void> {
		confirmDelete(async () => {
			try {
				const response = await slideshowApi.deleteSlideshow(slideshowId);
				showSuccess(response.message);
				await loadSavedSlideshows();
			} catch (e) {
				// Handle 404 errors (slideshow already deleted) gracefully
				if (e && typeof e === 'object' && 'message' in e) {
					const errorMessage = String(e.message);
					if (
						errorMessage.includes('404') ||
						errorMessage.includes('NOT_FOUND') ||
						errorMessage.includes('not found')
					) {
						await loadSavedSlideshows();
						showSuccess('Slideshow list updated successfully!');
						return;
					}
				}

				showErrorFromApi(
					`Error deleting slideshow: ${e instanceof Error ? e.message : 'Unknown error'}`
				);
			}
		});
	}

	function getImageUrl(image: MediaMetadata): string {
		return getMediaUrl(image.filename);
	}

	function selectAllImages(): void {
		selectedImages = images.map((img) => img.filename);
	}

	function clearSelection(): void {
		selectedImages = [];
	}

	// Multi-select functions for pool
	function getAvailableImages() {
		return images.filter((img) => !selectedImages.includes(img.filename));
	}

	function handlePoolImageClick(imageId: string): void {
		const newSelection = new SvelteSet(selectedInPool);
		if (newSelection.has(imageId)) {
			newSelection.delete(imageId);
		} else {
			newSelection.add(imageId);
		}
		selectedInPool = new SvelteSet(newSelection);
	}

	function addSelectedToSlideshow(): void {
		if (selectedInPool.size === 0) return;

		selectedImages = [...selectedImages, ...Array.from(selectedInPool)];
		selectedInPool = new SvelteSet();
	}

	function selectAllInPool(): void {
		const availableImages = getAvailableImages();
		selectedInPool = new SvelteSet(availableImages.map((img) => img.filename));
	}

	function clearPoolSelection(): void {
		selectedInPool = new SvelteSet<string>();
	}

	function invertPoolSelection(): void {
		const availableImages = getAvailableImages();
		const newSelection = new SvelteSet<string>();

		for (const img of availableImages) {
			if (!selectedInPool.has(img.filename)) {
				newSelection.add(img.filename);
			}
		}

		selectedInPool = newSelection;
	}

	// Drag and Drop Functions
	function handleDragStart(event: DragEvent, imageId: string): void {
		draggedImage = imageId;
		event.dataTransfer?.setData('text/plain', imageId);
		event.dataTransfer!.effectAllowed = 'move';
	}

	function handleDragOver(event: DragEvent, imageId: string): void {
		event.preventDefault();
		event.dataTransfer!.dropEffect = 'move';
		draggedOverImage = imageId;
	}

	function handleDragLeave(): void {
		draggedOverImage = null;
	}

	function handleDropOnImage(event: DragEvent, targetImageId: string, targetIndex: number): void {
		event.preventDefault();
		handleDropLogic(targetImageId, targetIndex);
		clearDragState();
	}

	function handleDropLogic(targetImageId: string, targetIndex: number): void {
		if (!draggedImage || draggedImage === targetImageId) {
			clearDragState();
			return;
		}

		const draggedIndex = selectedImages.indexOf(draggedImage);
		const targetImageExists = selectedImages.includes(targetImageId);

		if (draggedIndex !== -1 && targetImageExists) {
			const filtered = selectedImages.filter((id) => id !== draggedImage);

			const newTargetIndex = filtered.indexOf(targetImageId);

			const insertIndex = draggedIndex < targetIndex ? newTargetIndex + 1 : newTargetIndex;

			const newSelectedImages = [
				...filtered.slice(0, insertIndex),
				draggedImage,
				...filtered.slice(insertIndex)
			];

			selectedImages = newSelectedImages;
		} else if (draggedIndex !== -1) {
			selectedImages = selectedImages.filter((id) => id !== draggedImage);
		} else if (targetImageExists) {
			const newSelectedImages = [...selectedImages];
			newSelectedImages.splice(targetIndex, 0, draggedImage);
			selectedImages = newSelectedImages;
		} else {
			toggleImageSelection(draggedImage);
		}
	}

	function clearDragState(): void {
		draggedImage = null;
		draggedOverImage = null;
	}

	function handleDragEnd(): void {
		clearDragState();
	}

	function removeFromSlideshow(imageId: string): void {
		selectedImages = selectedImages.filter((id) => id !== imageId);
	}

	async function startEditSlideshow(slideshow: SlideshowSummary): Promise<void> {
		try {
			const completeSlideshow = await slideshowApi.getSlideshow(slideshow.id);

			if (!completeSlideshow) {
				throw new Error('Slideshow not found');
			}

			editingSlideshow = completeSlideshow;
			selectedImages = [...(completeSlideshow.media_ids || [])];
			interval = completeSlideshow.interval_seconds;
			slideshowName = completeSlideshow.name;
			shuffle = completeSlideshow.shuffle || false;
			currentWorkflow = 'create';
		} catch {
			showErrorFromApi('Error loading slideshow for editing. Please try again.');
		}
	}

	async function clearEditState(): Promise<void> {
		editingSlideshow = null;
		selectedImages = [];
		interval = 5;
		slideshowName = '';
		shuffle = false;

		if ($page.url.searchParams.has('edit')) {
			// eslint-disable-next-line svelte/no-navigation-without-resolve
			await goto('/slideshow', { replaceState: true });
		}
	}

	async function updateSlideshow(): Promise<void> {
		if (!editingSlideshow) return;

		try {
			await slideshowApi.updateSlideshow(editingSlideshow.id, {
				name: slideshowName,
				media_ids: [...selectedImages],
				interval_seconds: interval,
				loop_enabled: true,
				shuffle: shuffle
			});

			await loadSavedSlideshows();
			await clearEditState();
			currentWorkflow = 'saved';
			showSuccess('Slideshow updated successfully!');
		} catch {
			showErrorFromApi('Error updating slideshow. Please try again.');
		}
	}

	async function playSlideshow(slideshow: SlideshowSummary): Promise<void> {
		openDeviceModal('saved', slideshow);
	}

	async function playUnsavedSlideshow(): Promise<void> {
		openDeviceModal('unsaved');
	}
</script>

<div class="slideshow-manager">
	<!-- Header -->
	<div class="page-header">
		<div class="header-content">
			<div class="title-section">
				<h1>Slideshow Manager</h1>
				<p>Create and manage image presentations</p>
			</div>
			<div class="header-stats">
				<div class="stat-card">
					<div class="stat-value">{savedSlideshows.length}</div>
					<div class="stat-label">Saved</div>
				</div>
				<div class="stat-card">
					<div class="stat-value">{images.length}</div>
					<div class="stat-label">Images</div>
				</div>
				<div class="stat-card">
					<div class="stat-value">{selectedImages.length}</div>
					<div class="stat-label">Selected</div>
				</div>
			</div>
		</div>
	</div>

	<div class="main-content">
		{#if loading}
			<div class="loading-state">
				<div class="loading-spinner"></div>
				<p>Loading...</p>
			</div>
		{:else}
			<!-- Workflow Selector (only show when not editing) -->
			{#if !editingSlideshow}
				<div class="workflow-section">
					<div class="workflow-tabs">
						<button
							class="workflow-tab {currentWorkflow === 'saved' ? 'active' : ''}"
							onclick={() => (currentWorkflow = 'saved')}
						>
							<FolderOpen size={16} />
							<span>Saved ({savedSlideshows.length})</span>
						</button>
						<button
							class="workflow-tab {currentWorkflow === 'create' ? 'active' : ''}"
							onclick={() => (currentWorkflow = 'create')}
						>
							<Film size={16} />
							<span>Create</span>
						</button>
					</div>
				</div>
			{/if}

			{#if editingSlideshow}
				<div class="edit-mode-indicator">
					<div class="edit-breadcrumb">
						<button
							class="breadcrumb-link"
							onclick={async () => {
								await clearEditState();
								currentWorkflow = 'saved';
							}}>← Back to Saved Slideshows</button
						>
						<span class="breadcrumb-separator">/</span>
						<span class="breadcrumb-current">Edit: {editingSlideshow.name}</span>
					</div>
					<div class="edit-mode-badge">
						<Edit size={16} />
						<span>Edit Mode</span>
					</div>
				</div>
			{/if}

			{#if currentWorkflow === 'create' || editingSlideshow}
				<div class="create-section">
					<div class="section-header">
						<h2>
							{editingSlideshow
								? 'Edit Slideshow: ' + editingSlideshow.name
								: 'Create New Slideshow'}
						</h2>
					</div>

					<div class="settings-panel">
						<div class="settings-row">
							<div class="setting-group">
								<label for="slideshow-name">Name: *</label>
								<input
									id="slideshow-name"
									type="text"
									bind:value={slideshowName}
									placeholder="Slideshow name..."
									class="name-input"
									class:invalid={!slideshowName.trim()}
								/>
								{#if !slideshowName.trim()}
									<span class="field-error">Slideshow name is required</span>
								{/if}
							</div>
							<div class="setting-group">
								<label for="interval">Interval: {interval}s</label>
								<input id="interval" type="range" bind:value={interval} min="1" max="30" />
							</div>
							<div class="setting-group">
								<label for="mode">Mode:</label>
								<select
									id="mode"
									value={shuffle ? 'random' : 'sequential'}
									onchange={(e) => {
										shuffle = (e.target as HTMLSelectElement).value === 'random';
									}}
								>
									<option value="sequential">Sequential (in order)</option>
									<option value="random">Random (shuffle)</option>
								</select>
							</div>
						</div>
					</div>

					<div class="drag-drop-section">
						<div class="selection-header">
							<h3>Build Your Slideshow</h3>
							<div class="stats-row">
								<div class="stat-badge">
									<span class="stat-number">{getAvailableImages().length}</span>
									<span class="stat-label">Available</span>
								</div>
								<div class="stat-badge">
									<span class="stat-number">{selectedInPool.size}</span>
									<span class="stat-label">Selected</span>
								</div>
								<div class="stat-badge primary">
									<span class="stat-number">{selectedImages.length}</span>
									<span class="stat-label">In Slideshow</span>
								</div>
							</div>
						</div>

						{#if selectedImages.length === 0}
							<div class="field-error">Please add at least one image to the slideshow</div>
						{/if}

						{#if images.length === 0}
							<div class="empty-state">
								<Inbox size={64} />
								<p>No images found</p>
								<a href="/upload" class="btn-primary" rel="external">
									<CloudUpload size={20} />
									<span>Upload images</span>
								</a>
							</div>
						{:else}
							<div class="split-screen">
								<!-- Available Images Pool -->
								<div class="pool-panel">
									<div class="panel-header">
										<h4>Available Images</h4>
										<div class="pool-actions">
											<button
												class="btn-icon"
												onclick={selectAllInPool}
												title="Select all available images"
											>
												Select All
											</button>
											<button
												class="btn-icon"
												onclick={invertPoolSelection}
												title="Invert selection"
											>
												Invert
											</button>
											<button class="btn-icon" onclick={clearPoolSelection} title="Clear selection">
												Clear
											</button>
										</div>
									</div>

									{#if getAvailableImages().length === 0}
										<div class="pool-empty">
											<CheckCircle size={48} />
											<p>All images have been added to the slideshow!</p>
										</div>
									{:else}
										<div class="image-grid">
											{#each getAvailableImages() as image (image.filename)}
												<div
													class="grid-image"
													class:selected={selectedInPool.has(image.filename)}
													draggable="true"
													role="button"
													tabindex="0"
													aria-label={`Select ${image.filename} - click to select, drag to add to slideshow`}
													onclick={() => handlePoolImageClick(image.filename)}
													onkeydown={(e: KeyboardEvent) => {
														if (e.key === 'Enter' || e.key === ' ') {
															e.preventDefault();
															handlePoolImageClick(image.filename);
														}
													}}
													ondragstart={(e: DragEvent) => handleDragStart(e, image.filename)}
													ondragend={handleDragEnd}
												>
													<div class="grid-image-preview">
														<img src={getImageUrl(image)} alt={image.filename} />
														<div class="selection-checkbox">
															{#if selectedInPool.has(image.filename)}
																<CheckCircle size={20} />
															{/if}
														</div>
													</div>
													<div class="grid-image-name">{image.filename}</div>
													<div class="grid-image-size">
														{formatFileSize(image.size)}
													</div>
												</div>
											{/each}
										</div>
									{/if}

									<div class="panel-footer">
										<button
											class="btn-add-selected"
											onclick={addSelectedToSlideshow}
											disabled={selectedInPool.size === 0}
										>
											<span>Add Selected ({selectedInPool.size})</span>
										</button>
									</div>
								</div>

								<!-- Slideshow Timeline -->
								<div class="timeline-panel">
									<div class="panel-header">
										<h4>Slideshow Order</h4>
										<div class="timeline-actions">
											<button class="btn-icon" onclick={selectAllImages} title="Add all images">
												Add All
											</button>
											<button class="btn-icon" onclick={clearSelection} title="Clear slideshow">
												Clear
											</button>
										</div>
									</div>

									<div
										class="timeline-grid"
										role="region"
										aria-label="Slideshow timeline"
										ondragover={(e: DragEvent) => e.preventDefault()}
										ondrop={(e: DragEvent) => {
											e.preventDefault();
											if (draggedImage && !selectedImages.includes(draggedImage)) {
												selectedImages = [...selectedImages, draggedImage];
											}
											clearDragState();
										}}
									>
										{#if selectedImages.length === 0}
											<div class="timeline-empty">
												<FolderOpen size={48} />
												<p>Select images from the pool</p>
												<p class="timeline-hint">
													Click images or select multiple and click "Add Selected"
												</p>
											</div>
										{:else}
											<div class="timeline-images-grid">
												{#each selectedImages as imageId, index (imageId)}
													{@const image = images.find((img) => img.filename === imageId)}
													{#if image}
														<div
															class="timeline-grid-image"
															draggable="true"
															class:drag-over={draggedOverImage === imageId}
															role="button"
															tabindex="0"
															aria-label={`Timeline image ${index + 1}: ${image.filename} - drag to reorder`}
															ondragstart={(e: DragEvent) => handleDragStart(e, imageId)}
															ondragover={(e: DragEvent) => handleDragOver(e, imageId)}
															ondragleave={handleDragLeave}
															ondrop={(e: DragEvent) => handleDropOnImage(e, imageId, index)}
															ondragend={handleDragEnd}
														>
															<div class="timeline-grid-number">
																{index + 1}
															</div>
															<div class="timeline-grid-preview">
																<img src={getImageUrl(image)} alt={image.filename} />
															</div>
															<div class="timeline-grid-info">
																<div class="timeline-grid-name">
																	{image.filename}
																</div>
																<div class="timeline-grid-size">
																	{formatFileSize(image.size)}
																</div>
															</div>
															<button
																class="timeline-grid-remove"
																onclick={(e) => {
																	e.stopPropagation();
																	removeFromSlideshow(imageId);
																}}
																title="Remove from slideshow"
															>
																×
															</button>
														</div>
													{/if}
												{/each}
											</div>
										{/if}
									</div>
								</div>
							</div>
						{/if}
					</div>

					<div class="action-section">
						<div class="action-buttons">
							{#if editingSlideshow}
								<button
									class="btn-primary"
									onclick={updateSlideshow}
									disabled={selectedImages.length === 0 || !slideshowName.trim()}
									type="button"
								>
									<CheckCircle size={20} />
									<span>Update Slideshow</span>
								</button>
								<button class="btn-secondary" onclick={() => clearEditState()} type="button">
									Cancel
								</button>
							{:else}
								<button
									class="btn-play"
									onclick={playUnsavedSlideshow}
									disabled={selectedImages.length === 0}
									type="button"
								>
									<Play size={20} />
									<span>Start Slideshow</span>
								</button>
								<button
									class="btn-primary"
									onclick={saveSlideshow}
									disabled={selectedImages.length === 0 || !slideshowName.trim()}
									type="button"
								>
									<FolderOpen size={20} />
									<span>Save Slideshow</span>
								</button>
							{/if}
						</div>
					</div>
				</div>
			{/if}

			{#if currentWorkflow === 'saved'}
				<div class="saved-section">
					<div class="section-header">
						<h2>Saved Slideshows</h2>
					</div>

					{#if savedSlideshows.length === 0}
						<div class="empty-state">
							<Inbox size={64} />
							<h3>No saved slideshows</h3>
							<p>You haven't created any slideshows yet.</p>
							<button class="btn-primary" onclick={() => (currentWorkflow = 'create')}>
								<Film size={16} />
								<span>Create first slideshow</span>
							</button>
						</div>
					{:else}
						<div class="slideshow-grid">
							{#each savedSlideshows as slideshow (slideshow.id)}
								<div class="slideshow-card">
									<div class="slideshow-header">
										<h3>{slideshow.name}</h3>
										<div class="slideshow-badge">
											{slideshow.media_ids?.length || 0} Images
										</div>
									</div>

									<div class="slideshow-meta">
										<div class="meta-item">
											<span class="meta-label">Interval:</span>
											<span class="meta-value">{slideshow.interval_seconds}s</span>
										</div>
										{#if slideshow.shuffle}
											<div class="meta-item shuffle">
												<span class="meta-label">Mode:</span>
												<Shuffle size={16} />
												<span>Random</span>
											</div>
										{/if}
									</div>

									<div class="slideshow-date">
										Created: {new Date(slideshow.created_at).toLocaleDateString('en-US')}
									</div>

									<div class="slideshow-actions">
										<button
											class="btn-play"
											onclick={() => playSlideshow(slideshow)}
											title="Start slideshow"
										>
											<Play size={16} />
											<span>Play</span>
										</button>
										<button
											class="btn-edit"
											onclick={() => startEditSlideshow(slideshow)}
											title="Edit slideshow"
										>
											<Edit size={16} />
											<span>Edit</span>
										</button>
										<button
											class="btn-delete"
											onclick={() => deleteSlideshow(slideshow.id)}
											title="Delete slideshow"
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
			{/if}
		{/if}
	</div>
</div>

{#if showDeleteConfirm}
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
		<div class="modal-content" onclick={(e) => e.stopPropagation()} role="presentation">
			<h3>Confirm Deletion</h3>
			<p>Are you sure you want to delete this slideshow?</p>
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
	description={getModalDescription()}
	onConfirm={handleDeviceConfirm}
	onCancel={closeDeviceModal}
/>

<style>
	.slideshow-manager {
		padding: var(--space-8);
		max-width: 1600px;
		margin: 0 auto;
	}

	.page-header {
		background: var(--color-surface);
		padding: var(--space-8);
		border-radius: var(--radius-xl);
		margin-bottom: var(--space-8);
		box-shadow: var(--shadow-sm);
		border: 1px solid var(--border-light);
	}

	.header-content {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: var(--space-6);
	}

	.title-section h1 {
		font-size: var(--font-size-3xl);
		font-weight: var(--font-weight-bold);
		color: var(--color-text);
		margin: 0 0 var(--space-2) 0;
	}

	.title-section p {
		color: var(--color-text-muted);
		font-size: var(--font-size-base);
		margin: 0;
	}

	.header-stats {
		display: flex;
		gap: var(--space-4);
	}

	.stat-card {
		background: var(--color-bg-secondary);
		padding: var(--space-4) var(--space-6);
		border-radius: var(--radius-lg);
		text-align: center;
		min-width: 100px;
		border: 2px solid transparent;
		transition: all 0.2s ease;
	}

	.stat-card:hover {
		border-color: var(--color-primary);
		transform: translateY(-2px);
	}

	.stat-value {
		font-size: var(--font-size-2xl);
		font-weight: var(--font-weight-bold);
		color: var(--color-primary);
		display: block;
	}

	.stat-label {
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.main-content {
		min-height: 500px;
	}

	.loading-state {
		text-align: center;
		padding: var(--space-16);
		background: var(--color-surface);
		border-radius: var(--radius-xl);
		box-shadow: var(--shadow-sm);
	}

	.loading-spinner {
		width: 48px;
		height: 48px;
		border: 4px solid var(--border-light);
		border-top-color: var(--color-primary);
		border-radius: 50%;
		animation: spin 1s linear infinite;
		margin: 0 auto var(--space-4);
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	.workflow-section {
		margin-bottom: var(--space-8);
	}

	.workflow-tabs {
		display: flex;
		gap: var(--space-4);
	}

	.workflow-tab {
		flex: 1;
		padding: var(--space-4) var(--space-6);
		background: var(--color-surface);
		border: 2px solid var(--border-light);
		border-radius: var(--radius-lg);
		cursor: pointer;
		font-size: var(--font-size-base);
		font-weight: var(--font-weight-medium);
		color: var(--color-text-secondary);
		display: flex;
		align-items: center;
		justify-content: center;
		gap: var(--space-2);
		transition: all 0.2s ease;
	}

	.workflow-tab:hover {
		border-color: var(--color-primary);
		color: var(--color-primary);
	}

	.workflow-tab.active {
		background: var(--color-primary);
		color: white;
		border-color: var(--color-primary);
	}

	.edit-mode-indicator {
		background: var(--color-primary-50);
		border: 2px solid var(--color-primary);
		border-radius: var(--radius-lg);
		padding: var(--space-4) var(--space-6);
		margin-bottom: var(--space-8);
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.edit-breadcrumb {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.breadcrumb-link {
		background: none;
		border: none;
		color: var(--color-primary);
		cursor: pointer;
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-medium);
		padding: 0;
	}

	.breadcrumb-link:hover {
		text-decoration: underline;
	}

	.breadcrumb-separator {
		color: var(--color-text-muted);
	}

	.breadcrumb-current {
		color: var(--color-text);
		font-weight: var(--font-weight-medium);
	}

	.edit-mode-badge {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		background: var(--color-primary);
		color: white;
		padding: var(--space-2) var(--space-4);
		border-radius: var(--radius-md);
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-medium);
	}

	.create-section {
		background: var(--color-surface);
		border-radius: var(--radius-xl);
		padding: var(--space-8);
		box-shadow: var(--shadow-sm);
		border: 1px solid var(--border-light);
	}

	.section-header {
		margin-bottom: var(--space-6);
	}

	.section-header h2 {
		font-size: var(--font-size-2xl);
		font-weight: var(--font-weight-bold);
		color: var(--color-text);
		margin: 0;
	}

	.settings-panel {
		background: var(--color-bg-secondary);
		border-radius: var(--radius-lg);
		padding: var(--space-6);
		margin-bottom: var(--space-8);
	}

	.settings-row {
		display: grid;
		grid-template-columns: 2fr 1fr 1fr;
		gap: var(--space-6);
	}

	.setting-group {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}

	.setting-group label {
		font-weight: var(--font-weight-medium);
		color: var(--color-text);
		font-size: var(--font-size-sm);
	}

	.name-input {
		padding: var(--space-3) var(--space-4);
		border: 2px solid var(--border-light);
		border-radius: var(--radius-md);
		font-size: var(--font-size-base);
		background: var(--color-surface);
		color: var(--color-text);
		transition: all 0.2s ease;
	}

	.name-input:focus {
		outline: none;
		border-color: var(--color-primary);
		box-shadow: 0 0 0 3px rgb(5 150 105 / 10%);
	}

	.name-input.invalid {
		border-color: var(--color-danger);
	}

	.field-error {
		color: var(--color-danger);
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-medium);
	}

	.setting-group input[type='range'] {
		width: 100%;
		height: 8px;
		border-radius: var(--radius-md);
		background: var(--color-neutral-200);
		outline: none;
		-webkit-appearance: none;
		appearance: none;
	}

	.setting-group input[type='range']::-webkit-slider-thumb {
		-webkit-appearance: none;
		appearance: none;
		width: 20px;
		height: 20px;
		border-radius: 50%;
		background: var(--color-primary);
		cursor: pointer;
	}

	.setting-group input[type='range']::-moz-range-thumb {
		width: 20px;
		height: 20px;
		border-radius: 50%;
		background: var(--color-primary);
		cursor: pointer;
		border: none;
	}

	.setting-group select {
		padding: var(--space-3) var(--space-4);
		border: 2px solid var(--border-light);
		border-radius: var(--radius-md);
		font-size: var(--font-size-base);
		background: var(--color-surface);
		color: var(--color-text);
		cursor: pointer;
	}

	.setting-group select:focus {
		outline: none;
		border-color: var(--color-primary);
	}

	.drag-drop-section {
		margin-bottom: var(--space-8);
	}

	.selection-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: var(--space-6);
		flex-wrap: wrap;
		gap: var(--space-4);
	}

	.stats-row {
		display: flex;
		gap: var(--space-3);
	}

	.stat-badge {
		display: flex;
		flex-direction: column;
		align-items: center;
		padding: var(--space-2) var(--space-4);
		background: var(--color-bg-secondary);
		border-radius: var(--radius-md);
		border: 2px solid var(--border-light);
		min-width: 80px;
	}

	.stat-badge.primary {
		background: var(--color-primary-50);
		border-color: var(--color-primary);
	}

	.stat-number {
		font-size: var(--font-size-xl);
		font-weight: var(--font-weight-bold);
		color: var(--color-primary);
		line-height: 1;
	}

	.stat-badge.primary .stat-number {
		color: var(--color-primary);
	}

	.stat-label {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.5px;
		margin-top: var(--space-1);
	}

	/* Split Screen Layout */
	.split-screen {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: var(--space-6);
		align-items: start;
	}

	.pool-panel,
	.timeline-panel {
		background: var(--color-bg-secondary);
		border-radius: var(--radius-lg);
		padding: var(--space-4);
		display: flex;
		flex-direction: column;
		max-height: 700px;
	}

	.panel-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: var(--space-4);
		flex-wrap: wrap;
		gap: var(--space-2);
	}

	.panel-header h4 {
		font-size: var(--font-size-base);
		font-weight: var(--font-weight-semibold);
		color: var(--color-text);
		margin: 0;
	}

	.pool-actions,
	.timeline-actions {
		display: flex;
		gap: var(--space-2);
	}

	.btn-icon {
		padding: var(--space-1) var(--space-3);
		font-size: var(--font-size-xs);
		background: var(--color-surface);
		border: 1px solid var(--border-light);
		border-radius: var(--radius-sm);
		cursor: pointer;
		font-weight: var(--font-weight-medium);
		color: var(--color-text);
		transition: all 0.2s ease;
		white-space: nowrap;
	}

	.btn-icon:hover {
		background: var(--color-primary);
		color: white;
		border-color: var(--color-primary);
	}

	/* Pool Image Grid */
	.image-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
		gap: var(--space-2);
		overflow-y: auto;
		flex: 1;
		padding-right: var(--space-2);
	}

	.grid-image {
		background: var(--color-surface);
		border: 2px solid var(--border-light);
		border-radius: var(--radius-md);
		overflow: hidden;
		cursor: pointer;
		transition: all 0.2s ease;
		position: relative;
	}

	.grid-image:hover {
		border-color: var(--color-primary);
		transform: translateY(-2px);
		box-shadow: var(--shadow-sm);
	}

	.grid-image.selected {
		border-color: var(--color-primary);
		background: var(--color-primary-50);
	}

	.grid-image-preview {
		aspect-ratio: 1;
		overflow: hidden;
		background: var(--color-neutral-100);
		position: relative;
	}

	.grid-image-preview img {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}

	.selection-checkbox {
		position: absolute;
		top: var(--space-1);
		right: var(--space-1);
		width: 24px;
		height: 24px;
		background: var(--color-surface);
		border-radius: 50%;
		display: flex;
		align-items: center;
		justify-content: center;
		box-shadow: var(--shadow-sm);
		color: var(--color-primary);
	}

	.grid-image-name {
		padding: var(--space-2) var(--space-2) var(--space-1);
		font-size: var(--font-size-xs);
		color: var(--color-text);
		font-weight: var(--font-weight-medium);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.grid-image-size {
		padding: 0 var(--space-2) var(--space-2);
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	.pool-empty {
		text-align: center;
		padding: var(--space-8);
		background: var(--color-surface);
		border-radius: var(--radius-md);
		color: var(--color-text-muted);
	}

	.panel-footer {
		margin-top: var(--space-4);
		padding-top: var(--space-4);
		border-top: 1px solid var(--border-light);
	}

	.btn-add-selected {
		width: 100%;
		padding: var(--space-3);
		background: var(--color-primary);
		color: white;
		border: none;
		border-radius: var(--radius-md);
		cursor: pointer;
		font-weight: var(--font-weight-semibold);
		font-size: var(--font-size-sm);
		display: flex;
		align-items: center;
		justify-content: center;
		gap: var(--space-2);
		transition: all 0.2s ease;
	}

	.btn-add-selected:hover:not(:disabled) {
		background: var(--color-primary-hover);
		transform: translateY(-1px);
		box-shadow: var(--shadow-md);
	}

	.btn-add-selected:disabled {
		background: var(--color-neutral-300);
		cursor: not-allowed;
		transform: none;
	}

	.timeline-grid {
		flex: 1;
		overflow-y: auto;
		min-height: 300px;
		border: 2px dashed var(--border-light);
		border-radius: var(--radius-md);
		padding: var(--space-3);
		background: var(--color-surface);
	}

	.timeline-empty {
		text-align: center;
		padding: var(--space-8);
		color: var(--color-text-muted);
	}

	.timeline-empty p {
		margin: var(--space-2) 0;
	}

	.timeline-hint {
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
		font-style: italic;
	}

	.timeline-images-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
		gap: var(--space-2);
	}

	.timeline-grid-image {
		position: relative;
		background: var(--color-bg-secondary);
		border: 2px solid var(--border-light);
		border-radius: var(--radius-md);
		overflow: hidden;
		cursor: move;
		transition: all 0.2s ease;
	}

	.timeline-grid-image:hover {
		border-color: var(--color-primary);
		box-shadow: var(--shadow-sm);
	}

	.timeline-grid-image.drag-over {
		border-color: var(--color-primary);
		background: var(--color-primary-50);
	}

	.timeline-grid-number {
		position: absolute;
		top: var(--space-1);
		left: var(--space-1);
		width: 24px;
		height: 24px;
		background: var(--color-primary);
		color: white;
		border-radius: 50%;
		display: flex;
		align-items: center;
		justify-content: center;
		font-weight: var(--font-weight-bold);
		font-size: var(--font-size-xs);
		z-index: 1;
		box-shadow: var(--shadow-sm);
	}

	.timeline-grid-preview {
		aspect-ratio: 1;
		overflow: hidden;
		background: var(--color-neutral-100);
	}

	.timeline-grid-preview img {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}

	.timeline-grid-info {
		padding: var(--space-2);
	}

	.timeline-grid-name {
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-medium);
		color: var(--color-text);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.timeline-grid-size {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	.timeline-grid-remove {
		position: absolute;
		top: var(--space-1);
		right: var(--space-1);
		width: 24px;
		height: 24px;
		background: var(--color-danger);
		color: white;
		border: none;
		border-radius: 50%;
		cursor: pointer;
		font-size: var(--font-size-sm);
		line-height: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1;
		box-shadow: var(--shadow-sm);
		transition: all 0.2s ease;
	}

	.timeline-grid-remove:hover {
		background: var(--color-danger-hover);
		transform: scale(1.1);
	}

	.action-section {
		margin-top: var(--space-8);
	}

	.action-buttons {
		display: flex;
		gap: var(--space-4);
		justify-content: flex-end;
	}

	.btn-primary {
		background: var(--color-primary);
		color: white;
		padding: var(--space-3) var(--space-6);
		border: none;
		border-radius: var(--radius-md);
		cursor: pointer;
		font-weight: var(--font-weight-medium);
		font-size: var(--font-size-base);
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		transition: all 0.2s ease;
	}

	.btn-primary:hover:not(:disabled) {
		background: var(--color-primary-hover);
		transform: translateY(-2px);
		box-shadow: var(--shadow-md);
	}

	.btn-primary:disabled {
		background: var(--color-neutral-300);
		cursor: not-allowed;
		transform: none;
	}

	.btn-secondary {
		background: var(--color-secondary);
		color: white;
		padding: var(--space-3) var(--space-6);
		border: none;
		border-radius: var(--radius-md);
		cursor: pointer;
		font-weight: var(--font-weight-medium);
		font-size: var(--font-size-base);
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		transition: all 0.2s ease;
	}

	.btn-secondary:hover:not(:disabled) {
		background: var(--color-secondary-hover);
		transform: translateY(-2px);
	}

	.btn-play {
		background: var(--color-success);
		color: white;
		padding: var(--space-3) var(--space-6);
		border: none;
		border-radius: var(--radius-md);
		cursor: pointer;
		font-weight: var(--font-weight-medium);
		font-size: var(--font-size-base);
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		transition: all 0.2s ease;
	}

	.btn-play:hover:not(:disabled) {
		background: var(--color-success-hover);
		transform: translateY(-2px);
		box-shadow: var(--shadow-md);
	}

	.btn-play:disabled {
		background: var(--color-neutral-300);
		cursor: not-allowed;
		transform: none;
	}

	.btn-edit {
		background: var(--color-primary);
		color: white;
		padding: var(--space-2) var(--space-4);
		border: none;
		border-radius: var(--radius-md);
		cursor: pointer;
		font-weight: var(--font-weight-medium);
		font-size: var(--font-size-sm);
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		transition: all 0.2s ease;
	}

	.btn-edit:hover {
		background: var(--color-primary-hover);
		transform: translateY(-2px);
	}

	.btn-delete {
		background: var(--color-danger);
		color: white;
		padding: var(--space-2) var(--space-4);
		border: none;
		border-radius: var(--radius-md);
		cursor: pointer;
		font-weight: var(--font-weight-medium);
		font-size: var(--font-size-sm);
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		transition: all 0.2s ease;
	}

	.btn-delete:hover {
		background: var(--color-danger-hover);
		transform: translateY(-2px);
	}

	.saved-section {
		background: var(--color-surface);
		border-radius: var(--radius-xl);
		padding: var(--space-8);
		box-shadow: var(--shadow-sm);
		border: 1px solid var(--border-light);
	}

	.empty-state {
		text-align: center;
		padding: var(--space-16);
		background: var(--color-bg-secondary);
		border-radius: var(--radius-lg);
	}

	.empty-state h3 {
		font-size: var(--font-size-xl);
		color: var(--color-text);
		margin: var(--space-4) 0 var(--space-2) 0;
	}

	.empty-state p {
		color: var(--color-text-muted);
		margin-bottom: var(--space-6);
	}

	.empty-state .btn-primary {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
	}

	.slideshow-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
		gap: var(--space-6);
	}

	.slideshow-card {
		background: var(--color-surface);
		border: 2px solid var(--border-light);
		border-radius: var(--radius-lg);
		padding: var(--space-6);
		transition: all 0.2s ease;
	}

	.slideshow-card:hover {
		border-color: var(--color-primary);
		box-shadow: var(--shadow-md);
		transform: translateY(-4px);
	}

	.slideshow-header {
		display: flex;
		justify-content: space-between;
		align-items: start;
		margin-bottom: var(--space-4);
		gap: var(--space-3);
	}

	.slideshow-header h3 {
		font-size: var(--font-size-lg);
		font-weight: var(--font-weight-semibold);
		color: var(--color-text);
		margin: 0;
		flex: 1;
		word-wrap: break-word;
	}

	.slideshow-badge {
		background: var(--color-primary-100);
		color: var(--color-primary);
		padding: var(--space-1) var(--space-3);
		border-radius: var(--radius-md);
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-semibold);
		white-space: nowrap;
		flex-shrink: 0;
	}

	.slideshow-meta {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
		margin-bottom: var(--space-4);
	}

	.meta-item {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
	}

	.meta-item.shuffle {
		color: var(--color-primary);
	}

	.meta-label {
		font-weight: var(--font-weight-medium);
	}

	.meta-value {
		color: var(--color-text);
	}

	.slideshow-date {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		margin-bottom: var(--space-6);
	}

	.slideshow-actions {
		display: flex;
		gap: var(--space-3);
	}

	.slideshow-actions button {
		flex: 1;
		justify-content: center;
	}

	.modal-overlay {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background: rgba(0, 0, 0, 0.5);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: var(--z-modal);
		padding: var(--space-4);
	}

	.modal-content {
		background: var(--color-surface);
		border-radius: var(--radius-xl);
		padding: var(--space-8);
		max-width: 500px;
		width: 100%;
		box-shadow: var(--shadow-2xl);
	}

	.modal-content h3 {
		font-size: var(--font-size-xl);
		font-weight: var(--font-weight-bold);
		color: var(--color-text);
		margin: 0 0 var(--space-4) 0;
	}

	.modal-content p {
		color: var(--color-text-secondary);
		margin-bottom: var(--space-2);
	}

	.modal-warning {
		color: var(--color-danger);
		font-weight: var(--font-weight-medium);
		margin-top: var(--space-4);
		margin-bottom: var(--space-6);
	}

	.modal-actions {
		display: flex;
		gap: var(--space-4);
		justify-content: flex-end;
	}

	.btn-cancel {
		background: var(--color-neutral-200);
		color: var(--color-text);
		padding: var(--space-3) var(--space-6);
		border: none;
		border-radius: var(--radius-md);
		cursor: pointer;
		font-weight: var(--font-weight-medium);
		transition: all 0.2s ease;
	}

	.btn-cancel:hover {
		background: var(--color-neutral-300);
	}

	.btn-cancel + .btn-delete {
		margin-left: 0;
	}

	@media (max-width: 1024px) {
		.slideshow-manager {
			padding: var(--space-4);
		}

		.settings-row {
			grid-template-columns: 1fr;
		}

		.slideshow-grid {
			grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
		}

		.split-screen {
			grid-template-columns: 1fr;
			gap: var(--space-4);
		}

		.pool-panel,
		.timeline-panel {
			max-height: none;
		}

		.image-grid,
		.timeline-grid {
			max-height: 400px;
		}
	}

	@media (max-width: 768px) {
		.header-content {
			flex-direction: column;
			gap: var(--space-4);
			align-items: stretch;
		}

		.header-stats {
			justify-content: space-around;
		}

		.workflow-tabs {
			flex-direction: column;
		}

		.action-buttons {
			flex-direction: column;
		}

		.slideshow-actions {
			flex-direction: column;
		}

		.modal-actions {
			flex-direction: column-reverse;
		}

		.btn-cancel + .btn-delete {
			margin-left: 0;
			margin-top: var(--space-3);
		}

		.selection-header {
			flex-direction: column;
			align-items: stretch;
		}

		.stats-row {
			justify-content: center;
		}

		.pool-actions,
		.timeline-actions {
			flex-wrap: wrap;
		}

		.image-grid,
		.timeline-images-grid {
			grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
		}
	}

	@media (max-width: 480px) {
		.slideshow-manager {
			padding: var(--space-3);
		}

		.page-header {
			padding: var(--space-4);
		}

		.title-section h1 {
			font-size: var(--font-size-xl);
		}

		.title-section p {
			font-size: var(--font-size-sm);
		}

		.stat-card {
			min-width: 70px;
			padding: var(--space-2) var(--space-3);
		}

		.stat-value {
			font-size: var(--font-size-lg);
		}

		.workflow-tab {
			padding: var(--space-3) var(--space-4);
			font-size: var(--font-size-sm);
		}

		.edit-mode-indicator {
			flex-direction: column;
			gap: var(--space-3);
			text-align: center;
		}

		.create-section {
			padding: var(--space-4);
		}

		.section-header h2 {
			font-size: var(--font-size-lg);
		}

		.settings-panel {
			padding: var(--space-4);
		}

		.setting-group input[type='range'] {
			height: 6px;
		}

		.name-input,
		.setting-group select {
			padding: var(--space-2) var(--space-3);
			font-size: var(--font-size-sm);
		}

		.pool-panel,
		.timeline-panel {
			padding: var(--space-3);
		}

		.panel-header {
			flex-direction: column;
			align-items: stretch;
			gap: var(--space-2);
		}

		.btn-icon {
			padding: var(--space-2) var(--space-3);
			min-height: 36px;
		}

		.btn-add-selected {
			min-height: 44px;
		}

		.btn-play,
		.btn-edit,
		.btn-delete {
			flex: 1;
			justify-content: center;
			min-height: 44px;
		}

		.slideshow-grid {
			grid-template-columns: 1fr;
		}

		.slideshow-card {
			padding: var(--space-4);
		}

		.slideshow-header {
			flex-direction: column;
		}

		.slideshow-actions {
			margin-top: var(--space-3);
		}
	}

	@media (max-width: 390px) {
		.slideshow-manager {
			padding: var(--space-2);
		}

		.page-header {
			padding: var(--space-3);
		}

		.title-section h1 {
			font-size: var(--font-size-lg);
		}

		.header-stats {
			flex-wrap: wrap;
			gap: var(--space-2);
		}

		.stat-label {
			font-size: var(--font-size-xs);
		}

		.stat-badge {
			min-width: 60px;
			padding: var(--space-1) var(--space-2);
		}

		.stat-number {
			font-size: var(--font-size-base);
		}

		.image-grid,
		.timeline-images-grid {
			grid-template-columns: repeat(auto-fill, minmax(80px, 1fr));
			gap: var(--space-1);
		}

		.timeline-grid-number {
			width: 20px;
			height: 20px;
			font-size: 10px;
		}

		.modal-content {
			width: 95%;
			padding: var(--space-4);
		}
	}
</style>
