<script lang="ts">
	import { Monitor, Settings } from '@lucide/svelte';
	import { onMount } from 'svelte';
	import { playbackApi } from '$lib/api/playback';
	import { slideshowApi } from '$lib/api/slideshow';
	import { devices, type Device } from '$lib/stores/devices';
	import { showSuccess, showError } from '$lib/stores/toast';
	import type { SlideshowSummary, SlideshowResponse } from '$lib/api/types';

	let playbackClients = $derived(
		$devices.slice().sort((a, b) => a.display_name.localeCompare(b.display_name))
	);
	let selectedClient = $state<Device | null>(null);

	let savedSlideshows = $state<SlideshowSummary[]>([]);

	let showConfigModal = $state(false);
	let modalTitle = $state('');
	let modalClient = $state<Device | null>(null);

	let selectedSlideshowId = $state<string>('');
	let selectedSlideshow = $state<SlideshowResponse | null>(null);

	async function loadSlideshows() {
		try {
			savedSlideshows = await slideshowApi.getSlideshows();
		} catch {
			savedSlideshows = [];
		}
	}

	async function loadSlideshowDetails(id: string): Promise<SlideshowResponse | null> {
		try {
			return await slideshowApi.getSlideshow(id);
		} catch {
			return null;
		}
	}

	onMount(async () => {
		await loadSlideshows();
	});

	function selectClient(client: Device) {
		selectedClient = client;
	}

	function openConfigModal(client: Device) {
		modalClient = client;
		modalTitle = `Configure Autostart - ${client.display_name}`;
		selectedSlideshowId = '';
		selectedSlideshow = null;
		showConfigModal = true;
	}

	function closeConfigModal() {
		showConfigModal = false;
		modalClient = null;
		selectedSlideshowId = '';
		selectedSlideshow = null;
	}

	async function onSlideshowSelect() {
		if (selectedSlideshowId) {
			selectedSlideshow = await loadSlideshowDetails(selectedSlideshowId);
		} else {
			selectedSlideshow = null;
		}
	}

	async function saveAutostartConfig() {
		if (!modalClient || !selectedSlideshow) {
			showError('Please select a slideshow');
			return;
		}

		try {
			const mediaFiles = selectedSlideshow.media_ids || [];

			await playbackApi.updateAutostartConfig({
				target_device_ids: [modalClient.device_id],
				autostart_config: {
					enabled: true,
					name: selectedSlideshow.name,
					type: 'local',
					media_files: mediaFiles,
					interval_seconds: selectedSlideshow.interval_seconds,
					shuffle_enabled: selectedSlideshow.shuffle,
					scaling_mode: 'fit',
					loop_enabled: selectedSlideshow.loop_enabled
				}
			});

			showSuccess(`Autostart configured for ${modalClient.display_name}`);
			closeConfigModal();
		} catch {
			showError('Failed to configure autostart');
		}
	}

	function getConnectionStatus(client: Device): string {
		return client.isConnected ? 'Connected' : 'Disconnected';
	}
</script>

<svelte:head>
	<title>Autostart Configuration - Slideshow Stack</title>
</svelte:head>

<div class="autostart-page">
	<div class="page-header">
		<h1>Autostart Configuration</h1>
		<p>Configure which slideshows start automatically on each device</p>
	</div>

	<div class="main-grid">
		<div class="client-section">
			<div class="section-header">
				<h2>Devices</h2>
				<div class="device-stats">
					<div class="stat-item">
						<span class="stat-value">{playbackClients.length}</span>
						<span class="stat-label">Total</span>
					</div>
					<div class="stat-item">
						<span class="stat-value">{playbackClients.filter((c) => c.isConnected).length}</span>
						<span class="stat-label">Connected</span>
					</div>
				</div>
			</div>

			{#if playbackClients.length === 0}
				<div class="empty-state">
					<Monitor size={48} />
					<h3>No devices found</h3>
					<p>Make sure playback clients are running and accessible</p>
				</div>
			{:else}
				<div class="client-list">
					{#each playbackClients as client (client.device_id)}
						<button
							type="button"
							class="client-card"
							class:selected={selectedClient?.device_id === client.device_id}
							onclick={() => selectClient(client)}
						>
							<div class="client-info">
								<div class="client-avatar">
									<Monitor size={24} />
								</div>
								<div class="client-details">
									<h3>{client.display_name}</h3>
									<div class="client-meta">
										<span
											class="connection-badge {client.isConnected ? 'connected' : 'disconnected'}"
										>
											{getConnectionStatus(client)}
										</span>
									</div>
								</div>
							</div>

							<div class="client-actions">
								<div
									class="config-btn"
									role="button"
									tabindex="0"
									onclick={(e) => {
										e.stopPropagation();
										openConfigModal(client);
									}}
									onkeydown={(e: KeyboardEvent) => {
										if (e.key === 'Enter' || e.key === ' ') {
											e.stopPropagation();
											openConfigModal(client);
										}
									}}
								>
									<Settings size={18} />
									<span>Configure</span>
								</div>
							</div>
						</button>
					{/each}
				</div>
			{/if}
		</div>

		<div class="details-section">
			{#if selectedClient}
				<div class="client-details-panel">
					<div class="panel-header">
						<h2>{selectedClient.display_name}</h2>
						<div class="status-indicator" class:connected={selectedClient.isConnected}>
							{getConnectionStatus(selectedClient)}
						</div>
					</div>

					<div class="client-status">
						<div class="status-item">
							<span class="status-label">Device ID</span>
							<code>{selectedClient.device_id}</code>
						</div>
						<div class="status-item">
							<span class="status-label">Status</span>
							<span class="status-value">
								{selectedClient.status?.isRunning ? 'Running' : 'Idle'}
							</span>
						</div>
					</div>

					<button
						class="primary-btn"
						onclick={() => selectedClient && openConfigModal(selectedClient)}
					>
						<Settings size={20} />
						<span>Configure Autostart</span>
					</button>
				</div>
			{:else}
				<div class="no-selection">
					<Monitor size={64} />
					<h3>No Device Selected</h3>
					<p>Select a device to view and configure its autostart settings</p>
				</div>
			{/if}
		</div>
	</div>

	{#if showConfigModal && modalClient}
		<div
			class="modal-overlay"
			role="dialog"
			aria-modal="true"
			onclick={closeConfigModal}
			onkeydown={(e: KeyboardEvent) => {
				if (e.key === 'Escape') closeConfigModal();
			}}
			tabindex="-1"
		>
			<div class="modal" role="presentation" onclick={(e) => e.stopPropagation()}>
				<div class="modal-header">
					<h2>{modalTitle}</h2>
					<button class="close-btn" onclick={closeConfigModal}>×</button>
				</div>

				<div class="modal-body">
					<div class="form-group">
						<label for="slideshow-select">Select Slideshow</label>
						<select
							id="slideshow-select"
							bind:value={selectedSlideshowId}
							onchange={onSlideshowSelect}
						>
							<option value="">Choose a slideshow...</option>
							{#each savedSlideshows as slideshow (slideshow.id)}
								<option value={slideshow.id}>{slideshow.name}</option>
							{/each}
						</select>
					</div>

					{#if selectedSlideshow}
						<div class="slideshow-preview">
							<h3>{selectedSlideshow.name}</h3>

							<div class="slideshow-stats">
								<div class="stat">
									<span class="stat-label">Media Files</span>
									<span class="stat-value">{selectedSlideshow.media_ids?.length || 0}</span>
								</div>
								<div class="stat">
									<span class="stat-label">Interval</span>
									<span class="stat-value">{selectedSlideshow.interval_seconds || 0}s</span>
								</div>
								<div class="stat">
									<span class="stat-label">Shuffle</span>
									<span class="stat-value"
										>{selectedSlideshow.shuffle ? 'Enabled' : 'Disabled'}</span
									>
								</div>
							</div>

							<div class="media-list">
								<h4>Media Files</h4>
								<ul>
									{#each selectedSlideshow.media_ids || [] as filename (filename)}
										<li>{filename}</li>
									{/each}
								</ul>
							</div>
						</div>
					{/if}
				</div>

				<div class="modal-footer">
					<button class="secondary-btn" onclick={closeConfigModal}> Cancel </button>
					<button class="primary-btn" onclick={saveAutostartConfig} disabled={!selectedSlideshow}>
						Save Configuration
					</button>
				</div>
			</div>
		</div>
	{/if}
</div>

<style>
	.autostart-page {
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

	.page-header h1 {
		margin: 0 0 var(--space-2) 0;
		font-size: var(--font-size-3xl);
		font-weight: var(--font-weight-bold);
		color: var(--color-text);
	}

	.page-header p {
		margin: 0;
		color: var(--color-text-muted);
		font-size: var(--font-size-lg);
	}

	.main-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: var(--space-6);
	}

	.client-section {
		background: white;
		border-radius: var(--radius-xl);
		padding: var(--space-6);
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
	}

	.section-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: var(--space-6);
	}

	.section-header h2 {
		margin: 0;
		font-size: var(--font-size-xl);
		font-weight: var(--font-weight-semibold);
		color: var(--color-text);
	}

	.device-stats {
		display: flex;
		gap: var(--space-4);
	}

	.stat-item {
		text-align: center;
		padding: var(--space-2) var(--space-3);
		background: var(--color-bg-secondary);
		border-radius: var(--radius-lg);
	}

	.stat-value {
		display: block;
		font-size: var(--font-size-lg);
		font-weight: var(--font-weight-bold);
		color: var(--color-primary);
	}

	.stat-label {
		display: block;
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.empty-state {
		grid-column: 1 / -1;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-3);
		padding: var(--space-12);
		text-align: center;
		color: var(--color-text-muted);
	}

	.empty-state h3 {
		margin: 0;
		font-size: var(--font-size-lg);
		font-weight: var(--font-weight-semibold);
		color: var(--color-text);
	}

	.client-list {
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}

	.client-card {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: var(--space-4);
		background: var(--color-bg-secondary);
		border: 2px solid transparent;
		border-radius: var(--radius-lg);
		cursor: pointer;
		transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
	}

	.client-card:hover {
		border-color: var(--color-primary);
		transform: translateY(-1px);
		box-shadow: 0 4px 15px rgba(0, 0, 0, 0.1);
	}

	.client-card.selected {
		border-color: var(--color-primary);
		background: rgba(141, 198, 63, 0.05);
		box-shadow: 0 4px 15px rgba(141, 198, 63, 0.2);
	}

	.client-info {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		flex: 1;
		min-width: 0;
	}

	.client-avatar {
		width: 48px;
		height: 48px;
		background: var(--color-primary);
		color: white;
		border-radius: var(--radius-lg);
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.client-details {
		flex: 1;
		min-width: 0;
	}

	.client-details h3 {
		margin: 0 0 var(--space-1) 0;
		font-size: var(--font-size-base);
		font-weight: var(--font-weight-semibold);
		color: var(--color-text);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.client-meta {
		display: flex;
		gap: var(--space-2);
	}

	.connection-badge {
		padding: var(--space-1) var(--space-2);
		border-radius: var(--radius-md);
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-medium);
	}

	.connection-badge.connected {
		background: rgba(141, 198, 63, 0.1);
		color: var(--color-primary);
	}

	.connection-badge.disconnected {
		background: rgba(231, 76, 60, 0.1);
		color: var(--color-danger);
	}

	.client-actions {
		display: flex;
		gap: var(--space-2);
	}

	.config-btn {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		padding: var(--space-2) var(--space-3);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-md);
		background: white;
		color: var(--color-text);
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-medium);
		cursor: pointer;
		transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
	}

	.config-btn:hover {
		border-color: var(--color-primary);
		color: var(--color-primary);
	}

	.config-btn:hover {
		border-color: var(--color-primary);
		color: var(--color-primary);
	}

	.details-section {
		background: white;
		border-radius: var(--radius-xl);
		padding: var(--space-6);
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
	}

	.client-details-panel {
		display: flex;
		flex-direction: column;
		gap: var(--space-6);
	}

	.panel-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.panel-header h2 {
		margin: 0;
		font-size: var(--font-size-xl);
		font-weight: var(--font-weight-semibold);
		color: var(--color-text);
	}

	.status-indicator {
		padding: var(--space-2) var(--space-3);
		border-radius: var(--radius-lg);
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-medium);
		background: var(--color-bg-secondary);
		color: var(--color-text-muted);
	}

	.status-indicator.connected {
		background: rgba(141, 198, 63, 0.1);
		color: var(--color-primary);
	}

	.client-status {
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}

	.status-item {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: var(--space-3);
		background: var(--color-bg-secondary);
		border-radius: var(--radius-lg);
	}

	.status-label {
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
	}

	.status-value {
		font-weight: var(--font-weight-medium);
		color: var(--color-text);
	}

	.status-item code {
		background: var(--color-bg);
		padding: var(--space-1) var(--space-2);
		border-radius: var(--radius-md);
		font-family: var(--font-family-mono);
		font-size: var(--font-size-xs);
		word-break: break-all;
	}

	.no-selection {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-4);
		padding: var(--space-12);
		text-align: center;
	}

	.no-selection h3 {
		margin: 0;
		font-size: var(--font-size-xl);
		font-weight: var(--font-weight-semibold);
		color: var(--color-text);
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
		z-index: 999;
	}

	.modal {
		background: white;
		border-radius: var(--radius-xl);
		width: 90%;
		max-width: 600px;
		max-height: 80vh;
		overflow: hidden;
		display: flex;
		flex-direction: column;
		position: relative;
	}

	.modal-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: var(--space-6);
		border-bottom: 1px solid var(--color-border);
		flex-shrink: 0;
	}

	.modal-header h2 {
		margin: 0;
		font-size: var(--font-size-xl);
		font-weight: var(--font-weight-semibold);
		color: var(--color-text);
	}

	.close-btn {
		width: 32px;
		height: 32px;
		border: none;
		background: none;
		font-size: var(--font-size-2xl);
		color: var(--color-text-muted);
		cursor: pointer;
		border-radius: var(--radius-lg);
		transition: all 0.3s;
	}

	.close-btn:hover {
		background: var(--color-bg-secondary);
		color: var(--color-text);
	}

	.modal-body {
		flex: 1;
		overflow-y: auto;
		padding: var(--space-6);
		min-height: 0;
	}

	.form-group {
		margin-bottom: var(--space-4);
	}

	.form-group label {
		display: block;
		margin-bottom: var(--space-2);
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-semibold);
		color: var(--color-text);
	}

	.form-group select {
		width: 100%;
		padding: var(--space-3);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-lg);
		font-size: var(--font-size-base);
		background: white;
		color: var(--color-text);
	}

	.slideshow-preview {
		margin-top: var(--space-6);
		padding: var(--space-4);
		background: var(--color-bg-secondary);
		border-radius: var(--radius-lg);
	}

	.slideshow-preview h3 {
		margin: 0 0 var(--space-2) 0;
		font-size: var(--font-size-lg);
		font-weight: var(--font-weight-semibold);
		color: var(--color-text);
	}

	.slideshow-stats {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: var(--space-3);
		margin-bottom: var(--space-4);
	}

	.stat {
		display: flex;
		flex-direction: column;
		align-items: center;
		padding: var(--space-3);
		background: white;
		border-radius: var(--radius-lg);
	}

	.stat-label {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.stat-value {
		font-size: var(--font-size-lg);
		font-weight: var(--font-weight-bold);
		color: var(--color-primary);
	}

	.media-list h4 {
		margin: 0 0 var(--space-2) 0;
		font-size: var(--font-size-base);
		font-weight: var(--font-weight-semibold);
		color: var(--color-text);
	}

	.media-list ul {
		margin: 0;
		padding: 0;
		list-style: none;
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}

	.media-list li {
		padding: var(--space-2) var(--space-3);
		background: white;
		border-radius: var(--radius-md);
		font-size: var(--font-size-sm);
		color: var(--color-text);
	}

	.modal-footer {
		display: flex;
		justify-content: flex-end;
		gap: var(--space-3);
		padding: var(--space-6);
		border-top: 1px solid var(--color-border);
		flex-shrink: 0;
		pointer-events: auto;
	}

	.secondary-btn {
		padding: var(--space-3) var(--space-4);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-lg);
		background: white;
		color: var(--color-text);
		font-size: var(--font-size-base);
		font-weight: var(--font-weight-medium);
		cursor: pointer;
		transition: all 0.3s;
		pointer-events: auto;
	}

	.secondary-btn:hover {
		background: var(--color-bg-secondary);
	}

	.primary-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: var(--space-2);
		padding: var(--space-3) var(--space-4);
		border: none;
		border-radius: var(--radius-lg);
		background: var(--color-primary);
		color: white;
		font-size: var(--font-size-base);
		font-weight: var(--font-weight-semibold);
		cursor: pointer;
		transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
		pointer-events: auto;
	}

	.primary-btn:hover:not(:disabled) {
		transform: translateY(-1px);
		box-shadow: 0 4px 15px rgba(141, 198, 63, 0.3);
	}

	.primary-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
		pointer-events: none;
	}

	@media (max-width: 1024px) {
		.main-grid {
			grid-template-columns: 1fr;
		}
	}

	@media (max-width: 768px) {
		.autostart-page {
			padding: var(--space-4);
		}

		.page-header {
			padding: var(--space-4);
		}

		.page-header h1 {
			font-size: var(--font-size-2xl);
		}

		.page-header p {
			font-size: var(--font-size-sm);
		}

		.section-header {
			flex-direction: column;
			gap: var(--space-3);
			align-items: stretch;
		}

		.client-section,
		.details-section {
			padding: var(--space-4);
		}

		.client-card {
			flex-direction: column;
			gap: var(--space-3);
			align-items: stretch;
		}

		.client-actions {
			width: 100%;
		}

		.config-btn {
			width: 100%;
			justify-content: center;
			min-height: 44px;
		}

		.client-details-panel {
			gap: var(--space-4);
		}

		.panel-header {
			flex-direction: column;
			gap: var(--space-2);
			align-items: flex-start;
		}

		.modal {
			width: 95%;
			max-height: 90vh;
		}

		.modal-header {
			padding: var(--space-4);
		}

		.modal-body {
			padding: var(--space-4);
		}

		.modal-footer {
			padding: var(--space-4);
			flex-direction: column-reverse;
		}

		.modal-footer button {
			width: 100%;
		}

		.slideshow-stats {
			grid-template-columns: 1fr;
		}
	}

	@media (max-width: 480px) {
		.autostart-page {
			padding: var(--space-3);
		}

		.page-header {
			padding: var(--space-3);
			margin-bottom: var(--space-4);
		}

		.page-header h1 {
			font-size: var(--font-size-xl);
		}

		.page-header p {
			font-size: var(--font-size-xs);
		}

		.stat-item {
			padding: var(--space-1) var(--space-2);
		}

		.stat-value {
			font-size: var(--font-size-base);
		}

		.client-avatar {
			width: 40px;
			height: 40px;
		}

		.client-details h3 {
			font-size: var(--font-size-sm);
		}

		.client-info {
			flex-direction: column;
			align-items: flex-start;
		}

		.no-selection {
			padding: var(--space-8) var(--space-4);
		}

		.primary-btn {
			min-height: 48px;
		}
	}

	@media (max-width: 390px) {
		.page-header h1 {
			font-size: var(--font-size-lg);
		}

		.client-list {
			gap: var(--space-2);
		}

		.client-card {
			padding: var(--space-3);
		}
	}
</style>
