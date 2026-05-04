<script lang="ts">
	import { toast } from '$lib/stores/toast';
	import Toast from './Toast.svelte';
</script>

<div class="toast-container" role="region" aria-label="Notifications">
	{#each $toast as toastItem (toastItem.id)}
		<div data-toast-id={toastItem.id}>
			<Toast
				type={toastItem.type}
				message={toastItem.message}
				{...toastItem.duration !== undefined ? { duration: toastItem.duration } : {}}
				{...toastItem.dismissible !== undefined ? { dismissible: toastItem.dismissible } : {}}
				on:toast-hide={() => toast.remove(toastItem.id)}
			/>
		</div>
	{/each}
</div>

<style>
	.toast-container {
		position: fixed;
		top: var(--space-6);
		right: var(--space-6);
		z-index: 9999;
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
		max-width: 100vw;
		pointer-events: none;
	}

	.toast-container > * {
		pointer-events: auto;
	}

	/* Mobile adjustments */
	@media (max-width: 640px) {
		.toast-container {
			top: var(--space-4);
			right: var(--space-4);
			left: var(--space-4);
			width: auto;
		}
	}
</style>
