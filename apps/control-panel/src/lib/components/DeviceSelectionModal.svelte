<script lang="ts">
	import { devices } from '$lib/stores/devices';
	import { showWarning } from '$lib/stores/toast';
	import type { Device } from '$lib/stores/devices';

	interface Props {
		show: boolean;
		title?: string;
		description?: string;
		onConfirm: (targetDeviceIds: string[]) => void;
		onCancel: () => void;
	}

	let {
		show,
		title = 'Select Playback Device',
		description = 'Choose where to start the slideshow',
		onConfirm,
		onCancel
	}: Props = $props();

	let playbackClients = $derived<Device[]>([]);
	let selectedDeviceId = $state<string>('');
	let selectedDeviceIds = $state<string[]>([]);

	let canStart = $derived(selectedDeviceId === 'broadcast' || selectedDeviceIds.length > 0);

	$effect(() => {
		if (show) {
			resetSelection();
		}
	});

	$effect(() => {
		playbackClients = $devices
			.filter((device) => device.isConnected)
			.sort((a, b) => a.display_name.localeCompare(b.display_name));
	});

	function resetSelection(): void {
		selectedDeviceId = '';
		selectedDeviceIds = [];
	}

	function handleBroadcastChange(value: string): void {
		selectedDeviceId = value;
		if (value === 'broadcast') {
			selectedDeviceIds = [];
		}
	}

	function handleIndividualDeviceChange(deviceId: string, checked: boolean): void {
		selectedDeviceId = '';
		if (checked) {
			selectedDeviceIds = [...selectedDeviceIds, deviceId];
		} else {
			selectedDeviceIds = selectedDeviceIds.filter((id) => id !== deviceId);
		}
	}

	function handleConfirm(): void {
		if (!canStart) {
			showWarning('Please select a playback device');
			return;
		}
		const targetIds = selectedDeviceId === 'broadcast' ? [] : selectedDeviceIds;
		onConfirm(targetIds);
	}

	function handleOverlayClick(): void {
		onCancel();
	}

	function handleModalClick(event: MouseEvent): void {
		event.stopPropagation();
	}
</script>

{#if show}
	<div
		class="device-modal-overlay"
		role="dialog"
		aria-modal="true"
		onclick={handleOverlayClick}
		onkeydown={(e: KeyboardEvent) => {
			if (e.key === 'Escape') handleOverlayClick();
		}}
		tabindex="-1"
	>
		<div class="device-modal-content" role="presentation" onclick={handleModalClick}>
			<div class="device-modal-header">
				<h3>{title}</h3>
				<p>{description}</p>
			</div>

			<div class="device-list">
				<label
					class="device-option broadcast-option"
					class:selected={selectedDeviceId === 'broadcast'}
				>
					<input
						type="radio"
						name="playback-mode"
						value="broadcast"
						checked={selectedDeviceId === 'broadcast'}
						onchange={(e) => handleBroadcastChange(e.currentTarget.value)}
					/>
					<div class="device-info">
						<div class="device-name">Broadcast to All Devices</div>
						<div class="device-id">Send to all connected playback clients</div>
					</div>
				</label>

				<div class="device-divider">
					<span>OR select individual devices:</span>
				</div>

				{#if playbackClients.length === 0}
					<div class="device-modal-error">No playback devices connected.</div>
				{:else}
					{#each playbackClients as client (client.device_id)}
						<label
							class="device-option"
							class:selected={selectedDeviceIds.includes(client.device_id)}
						>
							<input
								type="checkbox"
								checked={selectedDeviceIds.includes(client.device_id)}
								onchange={(e) =>
									handleIndividualDeviceChange(client.device_id, e.currentTarget.checked)}
							/>
							<div class="device-info">
								<div class="device-name">{client.display_name}</div>
								<div class="device-id">{client.device_id}</div>
							</div>
						</label>
					{/each}
				{/if}
			</div>

			<div class="device-actions">
				<button class="btn-cancel" onclick={onCancel}>Cancel</button>
				<button class="btn-confirm" onclick={handleConfirm} disabled={!canStart}>
					Start Slideshow
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.device-modal-overlay {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background: rgba(0, 0, 0, 0.7);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
		animation: fadeIn 0.2s ease;
	}

	.device-modal-content {
		background: var(--color-bg-card);
		border-radius: var(--radius-xl);
		padding: var(--space-8);
		max-width: 500px;
		width: 90%;
		box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
		animation: slideUp 0.3s ease;
		border: 2px solid var(--color-primary);
	}

	.device-modal-header {
		text-align: center;
		margin-bottom: var(--space-6);
	}

	.device-modal-header h3 {
		margin: 0 0 var(--space-3) 0;
		color: var(--color-text);
		font-size: var(--font-size-2xl);
		font-weight: var(--font-weight-bold);
	}

	.device-modal-header p {
		margin: 0;
		color: var(--color-text-muted);
		font-size: var(--font-size-base);
	}

	.device-list {
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
		margin-bottom: var(--space-6);
	}

	.device-option {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		padding: var(--space-4);
		border: 2px solid var(--border-light);
		border-radius: var(--radius-lg);
		cursor: pointer;
		transition: all 0.2s ease;
		background: var(--color-bg-secondary);
	}

	.device-option:hover {
		border-color: var(--color-primary);
		background: var(--color-bg-card);
		transform: translateX(4px);
		box-shadow: 0 4px 12px rgba(141, 198, 63, 0.15);
	}

	.device-option.selected {
		border-color: var(--color-primary);
		background: var(--color-primary);
		color: var(--color-text-inverse);
		box-shadow: 0 4px 12px rgba(141, 198, 63, 0.3);
	}

	.device-option.selected .device-id {
		color: var(--color-text-inverse);
		opacity: 0.9;
	}

	.device-option.broadcast-option {
		border-color: var(--color-primary);
		background: rgba(141, 198, 63, 0.08);
		font-weight: var(--font-weight-semibold);
	}

	.device-option.broadcast-option.selected {
		background: var(--color-primary);
		color: var(--color-text-inverse);
	}

	.device-divider {
		display: flex;
		align-items: center;
		margin: var(--space-4) 0;
		color: var(--color-text-muted);
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-medium);
	}

	.device-divider::before,
	.device-divider::after {
		content: '';
		flex: 1;
		height: 1px;
		background: var(--border-light);
	}

	.device-divider span {
		padding: 0 var(--space-3);
		white-space: nowrap;
	}

	.device-option input[type='radio'],
	.device-option input[type='checkbox'] {
		width: 20px;
		height: 20px;
		cursor: pointer;
		accent-color: var(--color-primary);
		flex-shrink: 0;
	}

	.device-info {
		flex: 1;
	}

	.device-name {
		font-weight: var(--font-weight-semibold);
		color: var(--color-text);
		font-size: var(--font-size-base);
		margin-bottom: var(--space-1);
	}

	.device-id {
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
		font-family: monospace;
	}

	.device-actions {
		display: flex;
		gap: var(--space-3);
		justify-content: flex-end;
	}

	.btn-cancel {
		padding: var(--space-3) var(--space-6);
		background: var(--color-bg-tertiary);
		color: var(--color-text);
		border: 2px solid var(--border-light);
		border-radius: var(--radius-md);
		font-size: var(--font-size-base);
		font-weight: var(--font-weight-medium);
		cursor: pointer;
		transition: var(--transition-colors);
	}

	.btn-cancel:hover {
		background: var(--color-bg-secondary);
		border-color: var(--color-text);
	}

	.btn-confirm {
		padding: var(--space-3) var(--space-6);
		background: var(--color-primary);
		color: var(--color-text-inverse);
		border: none;
		border-radius: var(--radius-md);
		font-size: var(--font-size-base);
		font-weight: var(--font-weight-semibold);
		cursor: pointer;
		transition: var(--transition-colors);
	}

	.btn-confirm:hover:not(:disabled) {
		background: var(--color-primary-hover);
		transform: translateY(-1px);
		box-shadow: 0 4px 12px rgba(141, 198, 63, 0.3);
	}

	.btn-confirm:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.device-modal-error {
		background: rgba(231, 76, 60, 0.1);
		border: 2px solid var(--color-danger);
		border-radius: var(--radius-md);
		padding: var(--space-3) var(--space-4);
		color: var(--color-danger);
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-medium);
	}

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
			transform: translateY(20px);
			opacity: 0;
		}
		to {
			transform: translateY(0);
			opacity: 1;
		}
	}
</style>
