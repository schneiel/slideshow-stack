<script lang="ts">
	import { Monitor, Play, Pause, Square, SkipForward, SkipBack, Zap, Film } from '@lucide/svelte';
	import { playbackApi } from '$lib/api/playback';
	import { devices, type Device, type VideoStatus } from '$lib/stores/devices';
	import { showInfo } from '$lib/stores/toast';

	let loading = $state(false);

	let playbackClients = $state<Device[]>([]);

	$effect(() => {
		const unsubscribe = devices.subscribe((d) => {
			playbackClients = d.slice().sort((a, b) => a.display_name.localeCompare(b.display_name));
		});
		return unsubscribe;
	});

	let selectedDeviceId: string | null = $state(null);
	let selectedClient: Device | null = $derived(
		selectedDeviceId ? playbackClients.find((d) => d.device_id === selectedDeviceId) || null : null
	);

	let clientStatus = $derived({
		isRunning: selectedClient?.status?.isRunning || false,
		isPaused: selectedClient?.status?.isPaused || false,
		scalingMode: selectedClient?.status?.scalingMode || 'Unknown',
		currentImage: selectedClient?.status?.currentImage || 'None',
		imageIndex: selectedClient?.status?.imageIndex || 0,
		totalImages: selectedClient?.status?.totalImages || 0,
		slideshowName: selectedClient?.status?.slideshowName || 'None',
		windowSize: selectedClient?.status?.windowSize || { width: 0, height: 0 },
		lastUpdate: selectedClient?.status?.lastUpdate || new Date()
	});

	let videoStatus = $derived<VideoStatus | undefined>(selectedClient?.videoStatus);

	let isVideoActive = $derived(videoStatus?.isPlaying === true || videoStatus?.isPaused === true);
	let isSlideshowActive = $derived(clientStatus.isRunning || clientStatus.isPaused);
	let isAnyActive = $derived(isVideoActive || isSlideshowActive);

	let connectionStatus = $derived(playbackClients.length > 0 ? 'connected' : 'disconnected');

	async function executeCommand(commandFn: () => Promise<unknown>) {
		if (!selectedClient) return;
		commandFn().catch(() => {});
	}

	let commands = $state({
		play: async () => {
			executeCommand(async () => {
				const result = await playbackApi.startSlideshow({
					target_device_ids: [selectedClient!.device_id],
					name: 'Manual Slideshow',
					media_files: ['tv1.png', 'tv2.png', 'tv3.png', 'tv4.png'],
					interval_seconds: 5,
					shuffle_enabled: false,
					scaling_mode: 'fit',
					loop_enabled: true
				});
				showInfo(`Request to start slideshow sent`, 2000);
				return result;
			});
		},
		pause: async () => {
			executeCommand(() =>
				playbackApi.pauseSlideshow({
					target_device_ids: [selectedClient!.device_id]
				})
			);
		},
		resume: async () => {
			executeCommand(() =>
				playbackApi.resumeSlideshow({
					target_device_ids: [selectedClient!.device_id]
				})
			);
		},
		stop: async () => {
			executeCommand(() =>
				playbackApi.stopSlideshow({
					target_device_ids: [selectedClient!.device_id],
					graceful: true
				})
			);
		},
		next: async () => {
			executeCommand(() =>
				playbackApi.nextImage({
					target_device_ids: [selectedClient!.device_id]
				})
			);
		},
		previous: async () => {
			executeCommand(() =>
				playbackApi.previousImage({
					target_device_ids: [selectedClient!.device_id]
				})
			);
		},
		scaling: async () => {
			const scalingModeToNumber: Record<string, number> = {
				none: 0,
				fit: 1,
				fill: 2,
				stretch: 3
			};
			const currentModeStr = isVideoActive
				? (videoStatus?.scalingMode ?? 'fit')
				: (selectedClient?.status?.scalingMode ?? 'fit');
			const currentMode = scalingModeToNumber[currentModeStr] ?? 1;
			const nextMode = (currentMode + 1) % 4;
			executeCommand(() =>
				isVideoActive
					? playbackApi.setVideoScaling({
							target_device_ids: [selectedClient!.device_id],
							mode: nextMode
						})
					: playbackApi.setScaling({
							target_device_ids: [selectedClient!.device_id],
							mode: nextMode
						})
			);
		},
		videoStop: async () => {
			executeCommand(() =>
				playbackApi.stopVideo({
					target_device_ids: [selectedClient!.device_id]
				})
			);
		},
		videoPause: async () => {
			executeCommand(() =>
				playbackApi.pauseVideo({
					target_device_ids: [selectedClient!.device_id]
				})
			);
		},
		videoResume: async () => {
			executeCommand(() =>
				playbackApi.resumeVideo({
					target_device_ids: [selectedClient!.device_id]
				})
			);
		}
	});

	function formatLastUpdate(date: Date): string {
		const now = new Date();
		const diff = now.getTime() - date.getTime();

		if (diff < 1000) return 'just now';
		if (diff < 60000) return `${Math.floor(diff / 1000)}s ago`;
		if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
		return `${Math.floor(diff / 3600000)}h ago`;
	}
</script>

<svelte:head>
	<title>Playback Control - Slideshow Stack</title>
</svelte:head>

<div class="playback-control">
	<div class="page-header">
		<div class="header-content">
			<div class="title-section">
				<h1>Playback Control</h1>
				<p>Control slideshow playback across connected devices</p>
			</div>
			<div
				class="server-status"
				class:connected={connectionStatus === 'connected'}
				class:disconnected={connectionStatus === 'disconnected'}
			>
				<div class="status-dot"></div>
				<span>
					{connectionStatus === 'connected'
						? 'Connected'
						: connectionStatus === 'disconnected'
							? 'Disconnected'
							: 'Error'}
				</span>
			</div>
		</div>
	</div>

	<div class="main-grid">
		<!-- Device Selection Section -->
		<div class="device-section">
			<div class="section-header">
				<h2>Connected Devices</h2>
				<div class="device-stats">
					<div class="stat-item">
						<span class="stat-value">{playbackClients.length}</span>
						<span class="stat-label">Total</span>
					</div>
					<div class="stat-item">
						<span class="stat-value"
							>{playbackClients.filter((c) => c.status?.isRunning).length}</span
						>
						<span class="stat-label">Active</span>
					</div>
				</div>
			</div>

			{#if loading}
				<div class="loading-state">
					<div class="loading-spinner"></div>
					<p>Discovering devices...</p>
				</div>
			{:else if playbackClients.length === 0}
				<div class="empty-state">
					<Monitor size={48} />
					<h3>No devices found</h3>
					<p>Make sure playback clients are running and accessible on the network</p>
				</div>
			{:else}
				<div class="device-grid">
					{#each playbackClients as client (client.device_id)}
						<button
							class="device-card"
							class:selected={selectedDeviceId === client.device_id}
							class:running={client.status?.isRunning}
							onclick={() => (selectedDeviceId = client.device_id)}
						>
							<div class="card-header">
								<div class="device-avatar">
									<Monitor size={24} />
								</div>
								<div class="device-info">
									<h3>{client.display_name}</h3>
									<div class="device-address">
										ID: {client.device_id.substring(0, 8)}
									</div>
								</div>
								<div
									class="status-indicator"
									class:active={client.device_id === selectedClient?.device_id}
								></div>
							</div>

							<div class="device-status">
								<div class="status-item">
									<span class="status-label">Status</span>
									<span class="status-value">
										{#if client.videoStatus?.isPlaying}
											Video Playing
										{:else if client.videoStatus?.isPaused}
											Video Paused
										{:else if client.status?.isRunning}
											Running
										{:else if client.status?.isPaused}
											Paused
										{:else}
											Idle
										{/if}
									</span>
								</div>
								{#if client.videoStatus?.filename}
									<div class="status-item">
										<span class="status-label">Video</span>
										<span class="status-value video-filename">{client.videoStatus.filename}</span>
									</div>
								{:else}
									<div class="status-item">
										<span class="status-label">Images</span>
										<span class="status-value">{client.status?.totalImages || 0}</span>
									</div>
								{/if}
								<div class="status-item">
									<span class="status-label">Scaling</span>
									<span class="status-value">{client.status?.scalingMode || 'Unknown'}</span>
								</div>
								{#if client.status?.currentImage && client.status?.currentImage !== 'None' && !client.videoStatus?.filename}
									<div class="current-image">
										<span class="status-label">Current</span>
										<code>{client.status.currentImage.split('/').pop()}</code>
									</div>
								{/if}
							</div>
						</button>
					{/each}
				</div>
			{/if}
		</div>

		<div class="control-section">
			{#if selectedClient}
				{@const status = selectedClient.status}
				<div class="control-panel">
					<div class="panel-header">
						<h2>{selectedClient.display_name}</h2>
						<div class="connection-indicator" class:running={status?.isRunning}>
							<div class="connection-dot active"></div>
							<span>{status?.isRunning ? 'Running' : 'Idle'}</span>
						</div>
					</div>

					<div class="client-status-info">
						<div class="status-header">
							<h3>Current Status</h3>
							<div class="last-update">
								Updated {formatLastUpdate(status?.lastUpdate || new Date())}
							</div>
						</div>

						<div class="status-grid">
							{#if videoStatus?.filename}
								<div class="status-card wide video-status">
									<div class="status-icon video">
										<Film size={16} />
									</div>
									<div class="status-details">
										<span class="status-label">Video</span>
										<span class="status-value">
											{videoStatus.filename}
											{#if videoStatus.isPlaying}
												<span class="badge playing">Playing</span>
											{:else if videoStatus.isPaused}
												<span class="badge paused">Paused</span>
											{:else}
												<span class="badge">{videoStatus.status || 'Idle'}</span>
											{/if}
										</span>
									</div>
								</div>
							{:else}
								<div class="status-card">
									<div class="status-icon">
										{#if status?.isRunning}
											<Play size={16} />
										{:else}
											<Square size={16} />
										{/if}
									</div>
									<div class="status-details">
										<span class="status-label">Slideshow</span>
										<span class="status-value">{status?.slideshowName || 'None'}</span>
									</div>
								</div>

								{#if (status?.totalImages || 0) > 0}
									<div class="status-card wide">
										<div class="status-icon">
											<Zap size={16} />
										</div>
										<div class="status-details">
											<span class="status-label">Progress</span>
											<span class="status-value">
												Image {(status?.imageIndex || 0) + 1} of {status?.totalImages || 0}
												{#if status?.currentImage && status.currentImage !== 'None'}
													- <code>{status.currentImage}</code>
												{/if}
											</span>
										</div>
									</div>
								{/if}
							{/if}

							<div class="status-card">
								<div class="status-icon">
									<Zap size={16} />
								</div>
								<div class="status-details">
									<span class="status-label">Scaling Mode</span>
									<span class="status-value"
										>{isVideoActive
											? videoStatus?.scalingMode || 'Unknown'
											: status?.scalingMode || 'Unknown'}</span
									>
								</div>
							</div>
						</div>
					</div>

					<div class="controls-section">
						<h3>Playback Controls</h3>

						<div class="control-grid">
							<button
								class="control-btn stop-btn"
								onclick={isVideoActive ? commands.videoStop : commands.stop}
								disabled={!isAnyActive}
							>
								<Square size={20} />
								<span>Stop</span>
							</button>

							<button
								class="control-btn pause-resume-btn"
								class:paused={isVideoActive ? videoStatus?.isPaused : clientStatus.isPaused}
								class:not-paused={isVideoActive ? !videoStatus?.isPaused : !clientStatus.isPaused}
								onclick={isVideoActive
									? videoStatus?.isPaused
										? commands.videoResume
										: commands.videoPause
									: clientStatus.isPaused
										? commands.resume
										: commands.pause}
								disabled={!isAnyActive}
							>
								{#if isVideoActive ? videoStatus?.isPaused : clientStatus.isPaused}
									<Play size={20} />
									<span>Resume</span>
								{:else}
									<Pause size={20} />
									<span>Pause</span>
								{/if}
							</button>
						</div>

						<div class="navigation-controls">
							<button class="nav-btn" onclick={commands.previous} disabled={!isSlideshowActive}>
								<SkipBack size={16} />
								<span>Previous</span>
							</button>

							<button class="nav-btn" onclick={commands.next} disabled={!isSlideshowActive}>
								<SkipForward size={16} />
								<span>Next</span>
							</button>
						</div>

						<div class="action-controls">
							<button class="action-btn" onclick={commands.scaling} disabled={!isAnyActive}>
								<Zap size={20} />
								<span>Cycle Scaling</span>
							</button>
						</div>
					</div>
				</div>
			{:else}
				<div class="no-selection">
					<Monitor size={64} />
					<h3>No Device Selected</h3>
					<p>
						Select a device from the grid to control its playback and view detailed status
						information
					</p>
				</div>
			{/if}
		</div>
	</div>
</div>

<style>
	.playback-control {
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

	.server-status {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		padding: var(--space-3) var(--space-4);
		background: var(--color-bg-secondary);
		border-radius: var(--radius-lg);
		font-weight: var(--font-weight-medium);
		transition: var(--transition-base);
	}

	.server-status.connected {
		background: rgba(141, 198, 63, 0.1);
		color: var(--color-primary);
	}

	.server-status.disconnected {
		background: rgba(231, 76, 60, 0.1);
		color: var(--color-danger);
	}

	.status-dot {
		width: 8px;
		height: 8px;
		border-radius: var(--radius-full);
		background: currentColor;
		animation: pulse 2s infinite;
	}

	.main-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: var(--space-6);
		align-items: start;
	}

	.device-section {
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

	.device-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
		gap: var(--space-4);
	}

	.loading-state,
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

	.loading-spinner {
		width: 32px;
		height: 32px;
		border: 3px solid var(--color-bg-secondary);
		border-top: 3px solid var(--color-primary);
		border-radius: 50%;
		animation: spin 1s linear infinite;
	}

	.device-card {
		background: var(--color-bg-secondary);
		border: 2px solid transparent;
		border-radius: var(--radius-lg);
		padding: var(--space-4);
		cursor: pointer;
		transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
		position: relative;
		overflow: hidden;
	}

	.device-card::before {
		content: '';
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		height: 4px;
		background: var(--color-bg-secondary);
		transition: var(--transition-base);
	}

	.device-card:hover {
		transform: translateY(-2px);
		box-shadow: 0 8px 25px rgba(0, 0, 0, 0.15);
		border-color: var(--color-primary);
	}

	.device-card.selected {
		border-color: var(--color-primary);
		background: rgba(141, 198, 63, 0.05);
		box-shadow: 0 4px 15px rgba(141, 198, 63, 0.2);
	}

	.device-card.selected::before {
		background: var(--color-primary);
	}

	.device-card.running {
		border-left: 4px solid var(--color-primary);
	}

	.card-header {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		margin-bottom: var(--space-4);
	}

	.device-avatar {
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

	.device-info {
		flex: 1;
		min-width: 0;
	}

	.device-info h3 {
		margin: 0 0 var(--space-1) 0;
		font-size: var(--font-size-base);
		font-weight: var(--font-weight-semibold);
		color: var(--color-text);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.device-address {
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
	}

	.status-indicator {
		width: 12px;
		height: 12px;
		border-radius: var(--radius-full);
		background: var(--color-text-muted);
		transition: var(--transition-base);
	}

	.status-indicator.active {
		background: var(--color-primary);
		animation: pulse 2s infinite;
		box-shadow: 0 0 0 2px rgba(141, 198, 63, 0.3);
	}

	.device-status {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}

	.status-item {
		display: flex;
		justify-content: space-between;
		font-size: var(--font-size-sm);
	}

	.status-label {
		color: var(--color-text-muted);
	}

	.status-value {
		font-weight: var(--font-weight-medium);
		color: var(--color-text);
	}

	.current-image {
		margin-top: var(--space-2);
		padding-top: var(--space-2);
		border-top: 1px solid var(--color-border);
		display: flex;
		flex-direction: column;
		gap: var(--space-1);
	}

	.current-image code {
		background: var(--color-bg);
		padding: var(--space-2) var(--space-3);
		border-radius: var(--radius-md);
		font-family: var(--font-family-mono);
		font-size: var(--font-size-xs);
		word-break: break-all;
		color: var(--color-text);
	}

	.control-section {
		display: flex;
		flex-direction: column;
		gap: var(--space-6);
	}

	.control-panel {
		background: white;
		border-radius: var(--radius-xl);
		padding: var(--space-6);
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
	}

	.panel-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: var(--space-6);
	}

	.panel-header h2 {
		margin: 0;
		font-size: var(--font-size-xl);
		font-weight: var(--font-weight-semibold);
		color: var(--color-text);
	}

	.connection-indicator {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		padding: var(--space-2) var(--space-3);
		background: var(--color-bg-secondary);
		border-radius: var(--radius-lg);
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-medium);
	}

	.connection-dot {
		width: 8px;
		height: 8px;
		border-radius: var(--radius-full);
		background: var(--color-text-muted);
		transition: var(--transition-base);
	}

	.connection-dot.active {
		background: var(--color-primary);
		animation: pulse 2s infinite;
	}

	.connection-indicator.running {
		background: rgba(141, 198, 63, 0.1);
		color: var(--color-primary);
	}

	.client-status-info {
		margin-bottom: var(--space-6);
	}

	.status-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: var(--space-4);
	}

	.status-header h3 {
		margin: 0;
		font-size: var(--font-size-lg);
		font-weight: var(--font-weight-semibold);
		color: var(--color-text);
	}

	.last-update {
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
	}

	.status-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
		gap: var(--space-3);
	}

	.status-card {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		padding: var(--space-4);
		background: var(--color-bg-secondary);
		border-radius: var(--radius-lg);
		border-left: 3px solid var(--color-primary);
		transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
	}

	.status-card:hover {
		transform: translateY(-1px);
		box-shadow: 0 4px 15px rgba(0, 0, 0, 0.1);
	}

	.status-card.wide {
		grid-column: 1 / -1;
	}

	.status-icon {
		width: 32px;
		height: 32px;
		background: var(--color-primary);
		color: white;
		border-radius: var(--radius-lg);
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.status-details {
		flex: 1;
		min-width: 0;
	}

	.status-details .status-label {
		display: block;
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.05em;
		margin-bottom: var(--space-1);
	}

	.status-details .status-value {
		display: block;
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-medium);
		color: var(--color-text);
	}

	.status-details code {
		background: var(--color-bg);
		padding: var(--space-1) var(--space-2);
		border-radius: var(--radius-md);
		font-family: var(--font-family-mono);
		font-size: var(--font-size-xs);
		word-break: break-all;
		color: var(--color-text);
	}

	.controls-section {
		border-top: 1px solid var(--color-border);
		padding-top: var(--space-6);
	}

	.controls-section h3 {
		margin: 0 0 var(--space-4) 0;
		font-size: var(--font-size-lg);
		font-weight: var(--font-weight-semibold);
		color: var(--color-text);
	}

	.control-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: var(--space-3);
		margin-bottom: var(--space-6);
	}

	.control-btn {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-2);
		padding: var(--space-4);
		border: none;
		border-radius: var(--radius-lg);
		font-weight: var(--font-weight-medium);
		cursor: pointer;
		transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
		font-size: var(--font-size-sm);
		min-height: 80px;
		position: relative;
		overflow: hidden;
	}

	.control-btn::before {
		content: '';
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background: linear-gradient(
			135deg,
			rgba(255, 255, 255, 0.1) 0%,
			rgba(255, 255, 255, 0.05) 100%
		);
		opacity: 0;
		transition: opacity 0.3s;
	}

	.control-btn:hover:not(:disabled)::before {
		opacity: 1;
	}

	.control-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
		transform: none !important;
	}

	.control-btn span {
		position: relative;
		z-index: 1;
	}

	.stop-btn {
		background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%);
		color: white;
	}

	.stop-btn:hover:not(:disabled) {
		transform: translateY(-2px);
		box-shadow: 0 8px 25px rgba(239, 68, 68, 0.3);
	}

	.pause-resume-btn {
		background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
		color: white;
	}

	.pause-resume-btn.paused {
		background: linear-gradient(135deg, #10b981 0%, #059669 100%);
	}

	.pause-resume-btn.paused:hover:not(:disabled) {
		transform: translateY(-2px);
		box-shadow: 0 8px 25px rgba(16, 185, 129, 0.3);
	}

	.pause-resume-btn.not-paused {
		background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
	}

	.pause-resume-btn.not-paused:hover:not(:disabled) {
		transform: translateY(-2px);
		box-shadow: 0 8px 25px rgba(245, 158, 11, 0.3);
	}

	.navigation-controls {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: var(--space-3);
		margin-bottom: var(--space-6);
	}

	.nav-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: var(--space-2);
		padding: var(--space-3);
		border: 2px solid var(--color-border);
		border-radius: var(--radius-lg);
		background: white;
		color: var(--color-text);
		font-weight: var(--font-weight-medium);
		cursor: pointer;
		transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
	}

	.nav-btn:hover:not(:disabled) {
		border-color: var(--color-primary);
		color: var(--color-primary);
		transform: translateY(-1px);
		box-shadow: 0 4px 15px rgba(141, 198, 63, 0.2);
	}

	.nav-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.action-controls {
		display: flex;
		gap: var(--space-3);
	}

	.action-btn {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: var(--space-2);
		padding: var(--space-3);
		border: 2px solid var(--color-border);
		border-radius: var(--radius-lg);
		background: white;
		color: var(--color-text);
		font-weight: var(--font-weight-medium);
		cursor: pointer;
		transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
	}

	.action-btn:hover:not(:disabled) {
		border-color: var(--color-primary);
		color: var(--color-primary);
		transform: translateY(-1px);
		box-shadow: 0 4px 15px rgba(141, 198, 63, 0.2);
	}

	.action-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	/* No Selection State */
	.no-selection {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-4);
		padding: var(--space-12);
		text-align: center;
		background: white;
		border-radius: var(--radius-xl);
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
	}

	.no-selection h3 {
		margin: 0;
		font-size: var(--font-size-xl);
		font-weight: var(--font-weight-semibold);
		color: var(--color-text);
	}

	@keyframes pulse {
		0%,
		100% {
			opacity: 1;
		}
		50% {
			opacity: 0.5;
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

	@media (max-width: 1024px) {
		.main-grid {
			grid-template-columns: 1fr;
			gap: var(--space-4);
		}

		.device-grid {
			grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
		}
	}

	@media (max-width: 640px) {
		.playback-control {
			padding: var(--space-4);
		}

		.header-content {
			flex-direction: column;
			gap: var(--space-4);
			align-items: stretch;
		}

		.control-grid {
			grid-template-columns: 1fr;
		}

		.navigation-controls {
			grid-template-columns: 1fr;
		}

		.action-controls {
			flex-direction: column;
		}

		.device-grid {
			grid-template-columns: 1fr;
		}

		.section-header {
			flex-direction: column;
			gap: var(--space-3);
			align-items: stretch;
		}

		.device-stats {
			justify-content: center;
		}
	}

	@media (max-width: 480px) {
		.playback-control {
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

		.device-section,
		.control-panel {
			padding: var(--space-4);
		}

		.section-header h2 {
			font-size: var(--font-size-lg);
		}

		.device-card {
			padding: var(--space-3);
		}

		.card-header {
			flex-wrap: wrap;
		}

		.device-avatar {
			width: 40px;
			height: 40px;
		}

		.status-card {
			padding: var(--space-3);
		}

		.control-btn {
			min-height: 64px;
			padding: var(--space-3);
		}

		.nav-btn,
		.action-btn {
			min-height: 44px;
			font-size: var(--font-size-sm);
		}

		.status-grid {
			grid-template-columns: 1fr;
		}

		.status-card.wide {
			grid-column: 1;
		}
	}

	@media (max-width: 390px) {
		.playback-control {
			padding: var(--space-2);
		}

		.page-header {
			padding: var(--space-3);
		}

		.title-section h1 {
			font-size: var(--font-size-lg);
		}

		.stat-item {
			padding: var(--space-1) var(--space-2);
		}

		.stat-value {
			font-size: var(--font-size-base);
		}

		.panel-header {
			flex-direction: column;
			gap: var(--space-2);
			align-items: flex-start;
		}

		.status-header {
			flex-direction: column;
			gap: var(--space-2);
			align-items: flex-start;
		}
	}

	.status-card.video-status {
		border-left-color: #8b51a3;
	}

	.status-card.video-status .status-icon.video {
		background: linear-gradient(135deg, #8b51a3 0%, #6b3d8a 100%);
	}

	.badge {
		display: inline-block;
		padding: var(--space-1) var(--space-2);
		border-radius: var(--radius-md);
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-medium);
		margin-left: var(--space-2);
		background: var(--color-bg);
	}

	.video-filename {
		color: var(--color-primary);
		font-weight: 600;
	}

	.badge.playing {
		background: rgba(16, 185, 129, 0.2);
		color: #059669;
	}

	.badge.paused {
		background: rgba(245, 158, 11, 0.2);
		color: #d97706;
	}
</style>
