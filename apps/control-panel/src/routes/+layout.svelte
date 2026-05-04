<script lang="ts">
	import '../app.css';
	import '$lib/styles/tokens.css';
	import '$lib/styles/utilities.css';
	import { type Snippet } from 'svelte';
	import { page } from '$app/stores';
	import ToastContainer from '$lib/components/ToastContainer.svelte';
	import { FolderOpen, Upload, Film, Monitor, Zap, Menu } from '@lucide/svelte';

	interface Props {
		children: Snippet;
	}

	const { children }: Props = $props();

	let sidebarOpen = $state(false);

	function toggleSidebar() {
		sidebarOpen = !sidebarOpen;
	}

	function closeSidebar() {
		sidebarOpen = false;
	}
</script>

<ToastContainer />

<!-- Mobile Header -->
<header class="mobile-header">
	<button class="menu-toggle" onclick={toggleSidebar} aria-label="Toggle menu">
		<Menu size={24} />
	</button>
	<div class="mobile-brand">
		<Film size={20} />
		<span>Slideshow</span>
	</div>
</header>

<!-- Sidebar Overlay -->
{#if sidebarOpen}
	<div class="sidebar-overlay" onclick={closeSidebar} role="presentation"></div>
{/if}

<!-- Modern Sidebar -->
<aside class="sidebar" class:open={sidebarOpen}>
	<!-- Sidebar Header -->
	<div class="sidebar-header">
		<div class="brand-section">
			<div class="brand-icon">
				<Film size={24} />
			</div>
			<div class="brand-text">
				<h2>
					<span class="brand-title">Slideshow</span>
					<span class="brand-title-emphasis">Control Panel</span>
				</h2>
			</div>
		</div>
	</div>

	<!-- Navigation Menu -->
	<nav class="sidebar-nav">
		<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -->
		<div class="nav-section">
			<div class="section-title">Main</div>
			<!-- eslint-disable svelte/no-navigation-without-resolve -->
			<a href="/" class:active={$page.url.pathname === '/'} class="nav-link">
				<FolderOpen size={20} class="nav-icon" />
				<span class="nav-text">Media Library</span>
			</a>
			<a href="/upload" class:active={$page.url.pathname === '/upload'} class="nav-link">
				<Upload size={20} class="nav-icon" />
				<span class="nav-text">Upload</span>
			</a>
			<a href="/slideshow" class:active={$page.url.pathname === '/slideshow'} class="nav-link">
				<Film size={20} class="nav-icon" />
				<span class="nav-text">Slideshows</span>
			</a>
			<a href="/playback" class:active={$page.url.pathname === '/playback'} class="nav-link">
				<Monitor size={20} class="nav-icon" />
				<span class="nav-text">Playback Control</span>
			</a>
			<a href="/autostart" class:active={$page.url.pathname === '/autostart'} class="nav-link">
				<Zap size={20} class="nav-icon" />
				<span class="nav-text">Autostart</span>
			</a>
			<!-- eslint-enable svelte/no-navigation-without-resolve -->
		</div>
	</nav>

	<!-- Sidebar Footer -->
	<div class="sidebar-footer">
		<div class="footer-content"></div>
	</div>
</aside>

<!-- Main Content Area -->
<div class="main-content">
	{@render children()}
</div>

<style>
	/* Modern Sidebar Layout */
	.sidebar {
		position: fixed;
		top: 0;
		left: 0;
		height: 100vh;
		width: var(--sidebar-width);
		background: var(--sidebar-bg);
		color: var(--sidebar-text);
		box-shadow: var(--shadow-xl);
		z-index: var(--z-fixed);
		display: flex;
		flex-direction: column;
		transition: var(--transition-slow);
		overflow: hidden;
	}

	/* Sidebar Header */
	.sidebar-header {
		padding: var(--space-6) var(--space-4);
		border-bottom: 1px solid rgba(255, 255, 255, 0.1);
		display: flex;
		align-items: center;
		justify-content: space-between;
		background: rgba(0, 0, 0, 0.2);
	}

	.brand-section {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		flex: 1;
	}

	.brand-icon {
		width: 40px;
		height: 40px;
		display: flex;
		align-items: center;
		justify-content: center;
		background: var(--color-bg);
		border-radius: var(--radius-lg);
		box-shadow: var(--shadow-md);
		animation: brandFloat 3s ease-in-out infinite;
		color: var(--color-primary);
	}

	@keyframes brandFloat {
		0%,
		100% {
			transform: translateY(0px);
		}
		50% {
			transform: translateY(-2px);
		}
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

	@keyframes blink {
		0%,
		100% {
			opacity: 1;
		}
		50% {
			opacity: 0.3;
		}
	}

	.brand-text {
		flex: 1;
		color: var(--sidebar-text);
		transition: var(--transition-base);
	}

	.brand-text h2 {
		margin: 0;
		font-size: var(--font-size-xl);
		font-weight: var(--font-weight-bold);
	}

	.brand-title {
		color: var(--color-text-inverse);
	}

	.brand-title-emphasis {
		color: var(--color-primary);
	}

	/* Navigation */
	.sidebar-nav {
		flex: 1;
		padding: var(--space-4) 0;
		overflow-y: auto;
	}

	.nav-section {
		margin-bottom: var(--space-8);
	}

	.section-title {
		color: rgba(255, 255, 255, 0.4);
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-semibold);
		text-transform: uppercase;
		letter-spacing: 0.1em;
		padding: 0 var(--space-4);
		margin-bottom: var(--space-2);
		transition: var(--transition-base);
	}

	.sidebar-nav .nav-link {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		padding: var(--space-3) var(--space-4);
		color: rgba(255, 255, 255, 0.7);
		background: none;
		border: none;
		text-align: left;
		text-decoration: none;
		cursor: pointer;
		transition: var(--transition-fast);
		position: relative;
		margin: var(--space-1) var(--space-4);
		border-radius: var(--radius-md);
		width: calc(100% - var(--space-8));
	}

	.sidebar-nav .nav-link::before {
		content: '';
		position: absolute;
		left: 0;
		top: 50%;
		transform: translateY(-50%);
		width: 3px;
		height: 0;
		background: var(--color-primary);
		border-radius: 0 2px 2px 0;
		transition: height var(--transition-fast);
	}

	.sidebar-nav .nav-link:hover {
		background: var(--sidebar-hover-bg);
		color: var(--sidebar-text);
		transform: translateX(4px);
	}

	.sidebar-nav .nav-link.active {
		background: rgba(141, 198, 63, 0.2);
		color: var(--sidebar-text);
		font-weight: var(--font-weight-semibold);
	}

	.sidebar-nav .nav-link.active::before {
		height: 70%;
	}

	.nav-text {
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-medium);
		transition: var(--transition-base);
		white-space: nowrap;
	}

	/* Sidebar Footer */
	.sidebar-footer {
		padding: var(--space-4);
		border-top: 1px solid rgba(255, 255, 255, 0.1);
		background: rgba(0, 0, 0, 0.2);
	}

	/* Main Content */
	.main-content {
		margin-left: var(--sidebar-width);
		padding: var(--space-8);
		min-height: 100vh;
		background: var(--color-bg);
		transition: var(--transition-slow);
	}

	/* Mobile Header */
	.mobile-header {
		display: none;
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		height: 56px;
		background: var(--sidebar-bg);
		padding: 0 var(--space-4);
		align-items: center;
		gap: var(--space-3);
		z-index: var(--z-fixed);
		box-shadow: var(--shadow-md);
	}

	.menu-toggle {
		background: transparent;
		border: none;
		color: white;
		padding: var(--space-2);
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		min-width: 44px;
		min-height: 44px;
		border-radius: var(--radius-md);
	}

	.menu-toggle:hover {
		background: var(--sidebar-hover-bg);
	}

	.mobile-brand {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		color: white;
		font-weight: var(--font-weight-semibold);
		font-size: var(--font-size-lg);
	}

	/* Sidebar Overlay */
	.sidebar-overlay {
		display: none;
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background: rgba(0, 0, 0, 0.5);
		z-index: calc(var(--z-fixed) - 1);
		animation: fadeIn 0.2s ease;
	}

	@keyframes fadeIn {
		from {
			opacity: 0;
		}
		to {
			opacity: 1;
		}
	}

	/* Responsive Design */
	@media (max-width: 768px) {
		.mobile-header {
			display: flex;
		}

		.sidebar {
			position: fixed;
			top: 0;
			left: 0;
			height: 100vh;
			width: 280px;
			transform: translateX(-100%);
			transition: transform 0.3s ease;
			z-index: var(--z-fixed);
		}

		.sidebar.open {
			transform: translateX(0);
		}

		.sidebar-overlay {
			display: block;
		}

		.main-content {
			margin-left: 0;
			padding: var(--space-4);
			padding-top: calc(56px + var(--space-4));
		}

		.sidebar-header {
			padding: var(--space-4);
		}

		.brand-icon {
			width: 35px;
			height: 35px;
		}

		.brand-text h2 {
			font-size: var(--font-size-lg);
		}

		.nav-link {
			min-height: 48px;
			padding: var(--space-3) var(--space-4);
		}
	}

	@media (max-width: 390px) {
		.sidebar {
			width: 100%;
		}

		.main-content {
			padding: var(--space-3);
			padding-top: calc(56px + var(--space-3));
		}
	}

	/* Smooth scrollbar for sidebar */
	.sidebar-nav::-webkit-scrollbar {
		width: 4px;
	}

	.sidebar-nav::-webkit-scrollbar-track {
		background: rgba(255, 255, 255, 0.05);
	}

	.sidebar-nav::-webkit-scrollbar-thumb {
		background: rgba(255, 255, 255, 0.2);
		border-radius: var(--radius-sm);
	}

	.sidebar-nav::-webkit-scrollbar-thumb:hover {
		background: rgba(255, 255, 255, 0.3);
	}
</style>
