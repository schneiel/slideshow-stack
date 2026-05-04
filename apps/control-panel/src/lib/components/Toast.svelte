<script lang="ts">
	import { onMount, createEventDispatcher } from 'svelte';
	import { fly } from 'svelte/transition';
	import { CheckCircle, AlertCircle, XCircle, Info } from '@lucide/svelte';

	interface Props {
		type?: 'success' | 'error' | 'warning' | 'info';
		message: string;
		duration?: number;
		dismissible?: boolean;
	}

	const { type = 'info', message, duration = 4000, dismissible = true }: Props = $props();

	let visible = $state(true);
	let timeoutId: number | undefined;
	let hideTimeoutId: number | undefined;
	const dispatch = createEventDispatcher();

	onMount(() => {
		if (duration > 0) {
			timeoutId = window.setTimeout(() => {
				hide();
			}, duration);
		}

		return () => {
			if (timeoutId) window.clearTimeout(timeoutId);
			if (hideTimeoutId) window.clearTimeout(hideTimeoutId);
		};
	});

	function hide(): void {
		visible = false;
		if (hideTimeoutId) window.clearTimeout(hideTimeoutId);

		hideTimeoutId = window.setTimeout(() => {
			dispatch('toast-hide');
		}, 300);
	}

	function getTypeClass(): string {
		return `toast-${type}`;
	}
</script>

{#if visible}
	<div
		class="toast {getTypeClass()}"
		transition:fly={{ x: 300, duration: 300 }}
		role="alert"
		aria-live="polite"
	>
		<div class="toast-icon">
			{#if type === 'success'}
				<CheckCircle size={20} />
			{:else if type === 'error'}
				<AlertCircle size={20} />
			{:else if type === 'warning'}
				<AlertCircle size={20} />
			{:else}
				<Info size={20} />
			{/if}
		</div>
		<div class="toast-message">
			{message}
		</div>
		{#if dismissible}
			<button class="toast-close" onclick={hide} aria-label="Close notification">
				<XCircle size={16} />
			</button>
		{/if}
	</div>
{/if}

<style>
	.toast {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		padding: var(--space-4) var(--space-6);
		border-radius: var(--radius-lg);
		box-shadow: var(--shadow-lg);
		max-width: 500px;
		margin-bottom: var(--space-3);
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-medium);
		color: var(--color-text-inverse);
		position: relative;
		animation: slideIn 0.3s ease-out;
	}

	.toast-icon {
		width: 20px;
		height: 20px;
		flex-shrink: 0;
	}

	.toast-message {
		flex: 1;
		word-break: break-word;
	}

	.toast-close {
		background: transparent;
		border: none;
		color: var(--color-text-inverse);
		cursor: pointer;
		padding: 0;
		width: 24px;
		height: 24px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: var(--radius-md);
		opacity: 0.8;
		transition: var(--transition-fast);
		flex-shrink: 0;
	}

	.toast-close:hover {
		opacity: 1;
		background: rgba(255, 255, 255, 0.1);
	}

	/* Type-specific styles */
	.toast-success {
		background: linear-gradient(135deg, var(--color-success) 0%, var(--color-success-dark) 100%);
		border: 1px solid rgba(16, 185, 129, 0.3);
	}

	.toast-error {
		background: linear-gradient(135deg, var(--color-danger) 0%, var(--color-danger-dark) 100%);
		border: 1px solid rgba(239, 68, 68, 0.3);
	}

	.toast-warning {
		background: linear-gradient(135deg, var(--color-warning) 0%, var(--color-warning-dark) 100%);
		border: 1px solid rgba(245, 158, 11, 0.3);
	}

	.toast-info {
		background: linear-gradient(135deg, var(--color-primary) 0%, var(--color-primary-hover) 100%);
		border: 1px solid rgba(141, 198, 63, 0.3);
	}

	@keyframes slideIn {
		from {
			transform: translateX(100%);
			opacity: 0;
		}
		to {
			transform: translateX(0);
			opacity: 1;
		}
	}

	/* Responsive */
	@media (max-width: 640px) {
		.toast {
			max-width: calc(100vw - 2rem);
			margin: 0 var(--space-4) var(--space-3);
		}
	}
</style>
