import type { Page, Navigation } from '@sveltejs/kit';
import { readable } from 'svelte/store';

declare module '$app/stores' {
	const page: readable<Page>;
	const navigating: readable<Navigation>;
	const updated: readable<boolean>;
	export { page, navigating, updated };
}
