<script lang="ts">
	import { user, logout, isLoading } from '$lib/auth.js';
	import { page } from '$app/stores';
	import { derived } from 'svelte/store';
	import { onMount } from 'svelte';
	import { unreadCount, isNotificationPanelOpen, fetchUnreadCount, fetchNotifications, toggleNotificationPanel, closeNotificationPanel } from '$lib/notifications.js';
	import NotificationPanel from './NotificationPanel.svelte';

	let showMobileMenu = false;
	let showUserMenu = false;

	// Close menus when clicking outside
	function handleOutsideClick() {
		showMobileMenu = false;
		showUserMenu = false;
		closeNotificationPanel();
	}

	// Create a derived store for current path that only updates when path changes
	const currentPath = derived(page, ($page) => $page.url.pathname);

	// Create individual derived stores for each tab's active state
	// These will only update when the specific tab's state changes
	const isSignupActive = derived(currentPath, ($path) => $path === '/signup');
	const isLoginActive = derived(currentPath, ($path) => $path === '/login');
	const isDashboardActive = derived(currentPath, ($path) => $path === '/dashboard');
	const isFindActive = derived(currentPath, ($path) => $path === '/find');
	const isExploreActive = derived(currentPath, ($path) => $path === '/explore');
	const isPublicationActive = derived(currentPath, ($path) => $path === '/publication');

	// Helper function for mobile menu (since we can't use derived stores in class:conditions)
	function isActive(path: string): boolean {
		return $currentPath === path;
	}

	function toggleMobileMenu() {
		showMobileMenu = !showMobileMenu;
		showUserMenu = false;
	}

	function toggleUserMenu() {
		showUserMenu = !showUserMenu;
		showMobileMenu = false;
	}

	async function handleLogout() {
		showUserMenu = false;
		await logout();
	}

	// Poll for new notifications every 30 seconds when user is logged in
	onMount(() => {
		let interval;
		
		const setupPolling = () => {
			if ($user) {
				// Initial fetch
				fetchNotifications().catch(console.error);
				
				// Set up polling
				interval = setInterval(() => {
					fetchNotifications().catch(console.error);
				}, 30000); // 30 seconds
			}
		};
		
		// Set up initial polling
		setupPolling();
		
		// Re-setup polling when user state changes
		const unsubscribeUser = user.subscribe(() => {
			if (interval) {
				clearInterval(interval);
			}
			setupPolling();
		});
		
		return () => {
			if (interval) {
				clearInterval(interval);
			}
			unsubscribeUser();
		};
	});

	function handleNotificationClick(event) {
		// Check if panel is currently open before toggling
		const wasOpen = $isNotificationPanelOpen;
		
		// Fetch notifications when panel is opened
		fetchNotifications().catch(console.error);
		toggleNotificationPanel();
		showUserMenu = false;
		
		// Only remove focus when closing the panel
		if (wasOpen) {
			// Find the button element (traverse up from the clicked element)
			const buttonElement = event.target.closest('button');
			if (buttonElement) {
				buttonElement.blur();
			}
		}
	}
</script>

<svelte:window on:click={handleOutsideClick} />

<nav class="bg-gray-800">
	<div class="mx-auto max-w-7xl px-2 sm:px-6 lg:px-8">
		<div class="relative flex h-16 items-center justify-between">
			<!-- Mobile menu button -->
			<div class="absolute inset-y-0 left-0 flex items-center sm:hidden">
				<button
					type="button"
					on:click|stopPropagation={toggleMobileMenu}
					class="relative inline-flex items-center justify-center rounded-md p-2 text-gray-400 hover:bg-gray-700 hover:text-white focus:outline-none focus:ring-2 focus:ring-inset focus:ring-white"
					aria-controls="mobile-menu"
					aria-expanded="false"
				>
					<span class="absolute -inset-0.5"></span>
					<span class="sr-only">Open main menu</span>
					<!-- Hamburger icon -->
					<svg
						class="block h-6 w-6"
						fill="none"
						viewBox="0 0 24 24"
						stroke-width="1.5"
						stroke="currentColor"
						aria-hidden="true"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5"
						/>
					</svg>
				</button>
			</div>

			<!-- Logo and main navigation -->
			<div class="flex flex-1 items-center justify-center sm:items-stretch sm:justify-start">
				<div class="flex shrink-0 items-center">
					<a href="/" class="px-3 py-1 text-lg font-bold text-green-300">
						glad
					</a>
				</div>
				
				<!-- Desktop navigation -->
				<div class="hidden sm:ml-6 sm:block">
					<div class="flex space-x-4">
						<!-- Render all items, control visibility with CSS classes -->
						<a
							href="/signup"
							class="rounded-md px-3 py-2 text-sm font-medium"
							class:hidden={$isLoading || $user}
							class:bg-gray-900={$isSignupActive}
							class:text-white={$isSignupActive}
							class:text-gray-300={!$isSignupActive}
							class:hover:bg-gray-700={!$isSignupActive}
							class:hover:text-white={!$isSignupActive}
						>
							Sign up
						</a>
						<a
							href="/login"
							class="rounded-md px-3 py-2 text-sm font-medium"
							class:hidden={$isLoading || $user}
							class:bg-gray-900={$isLoginActive}
							class:text-white={$isLoginActive}
							class:text-gray-300={!$isLoginActive}
							class:hover:bg-gray-700={!$isLoginActive}
							class:hover:text-white={!$isLoginActive}
						>
							Login
						</a>
						<a
							href="/dashboard"
							class="rounded-md px-3 py-2 text-sm font-medium"
							class:hidden={$isLoading || !$user}
							class:bg-gray-900={$isDashboardActive}
							class:text-white={$isDashboardActive}
							class:text-gray-300={!$isDashboardActive}
							class:hover:bg-gray-700={!$isDashboardActive}
							class:hover:text-white={!$isDashboardActive}
						>
							Dashboard
						</a>
						<a
							href="/find"
							class="rounded-md px-3 py-2 text-sm font-medium"
							class:hidden={$isLoading || !$user}
							class:bg-gray-900={$isFindActive}
							class:text-white={$isFindActive}
							class:text-gray-300={!$isFindActive}
							class:hover:bg-gray-700={!$isFindActive}
							class:hover:text-white={!$isFindActive}
						>
							Find Controls
						</a>
						<a
							href="/explore"
							class="rounded-md px-3 py-2 text-sm font-medium"
							class:hidden={$isLoading}
							class:bg-gray-900={$isExploreActive}
							class:text-white={$isExploreActive}
							class:text-gray-300={!$isExploreActive}
							class:hover:bg-gray-700={!$isExploreActive}
							class:hover:text-white={!$isExploreActive}
						>
							Explore
						</a>
						<a
							href="/publication"
							class="rounded-md px-3 py-2 text-sm font-medium"
							class:hidden={$isLoading}
							class:bg-gray-900={$isPublicationActive}
							class:text-white={$isPublicationActive}
							class:text-gray-300={!$isPublicationActive}
							class:hover:bg-gray-700={!$isPublicationActive}
							class:hover:text-white={!$isPublicationActive}
						>
							Publication
						</a>
					</div>
				</div>
			</div>

			<!-- User menu (when logged in) -->
			{#if $user}
				<div class="absolute inset-y-0 right-0 flex items-center pr-2 sm:static sm:inset-auto sm:ml-6 sm:pr-0">
					<!-- Notifications button -->
					<div class="relative">
						<button
							type="button"
							on:click|stopPropagation={handleNotificationClick}
							class="relative rounded-full bg-gray-800 p-1 text-gray-400 hover:text-white focus:outline-none focus:ring-2 focus:ring-white focus:ring-offset-2 focus:ring-offset-gray-800"
						>
							<span class="absolute -inset-1.5"></span>
							<span class="sr-only">View notifications</span>
							<svg
								class="h-6 w-6"
								fill="none"
								viewBox="0 0 24 24"
								stroke-width="1.5"
								stroke="currentColor"
								aria-hidden="true"
							>
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									d="M14.857 17.082a23.848 23.848 0 0 0 5.454-1.31A8.967 8.967 0 0 1 18 9.75V9A6 6 0 0 0 6 9v.75a8.967 8.967 0 0 1-2.312 6.022c1.733.64 3.56 1.085 5.455 1.31m5.714 0a24.255 24.255 0 0 1-5.714 0m5.714 0a3 3 0 1 1-5.714 0"
								/>
							</svg>
							
							<!-- Unread count badge -->
							{#if $unreadCount > 0}
								<span class="absolute -top-1 -right-1 flex h-4 w-4 items-center justify-center rounded-full bg-red-600 text-xs font-medium text-white">
									{$unreadCount > 9 ? '9+' : $unreadCount}
								</span>
							{/if}
						</button>
						
						<!-- Notification Panel -->
						<NotificationPanel isOpen={$isNotificationPanelOpen} />
					</div>

					<!-- Profile dropdown -->
					<div class="relative ml-3">
						<div>
							<button
								type="button"
								on:click|stopPropagation={toggleUserMenu}
								class="rounded-md px-3 py-2 text-sm font-medium text-gray-300 hover:bg-gray-700 hover:text-white"
								id="user-menu-button"
								aria-expanded="false"
								aria-haspopup="true"
							>
								<span class="absolute -inset-1.5"></span>
								<span class="sr-only">Open user menu</span>
								{$user.username}
							</button>
						</div>
						
						{#if showUserMenu}
							<div
								class="absolute right-0 z-10 mt-2 w-48 origin-top-right rounded-md bg-white py-1 shadow-lg ring-1 ring-black/5 focus:outline-none"
								role="menu"
								aria-orientation="vertical"
								aria-labelledby="user-menu-button"
								tabindex="-1"
							>
								<a
									href="/settings"
									class="block px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
									role="menuitem"
									tabindex="-1"
								>
									Settings
								</a>
								<button
									on:click={handleLogout}
									class="block w-full px-4 py-2 text-left text-sm text-gray-700 hover:bg-gray-100"
									role="menuitem"
									tabindex="-1"
								>
									Logout
								</button>
							</div>
						{/if}
					</div>
				</div>
			{/if}
		</div>
	</div>

	<!-- Mobile menu -->
	{#if showMobileMenu}
		<div class="sm:hidden" id="mobile-menu">
			<div class="space-y-1 px-2 pb-3 pt-2">
				{#if !$isLoading && !$user}
					<!-- Not logged in -->
					<a
						href="/signup"
						class="rounded-md px-3 py-2 text-base font-medium block"
						class:bg-gray-900={isActive('/signup')}
						class:text-white={isActive('/signup')}
						class:text-gray-300={!isActive('/signup')}
						class:hover:bg-gray-700={!isActive('/signup')}
						class:hover:text-white={!isActive('/signup')}
					>
						Sign up
					</a>
					<a
						href="/login"
						class="rounded-md px-3 py-2 text-base font-medium block"
						class:bg-gray-900={isActive('/login')}
						class:text-white={isActive('/login')}
						class:text-gray-300={!isActive('/login')}
						class:hover:bg-gray-700={!isActive('/login')}
						class:hover:text-white={!isActive('/login')}
					>
						Login
					</a>
				{/if}
				
				{#if !$isLoading && $user}
					<!-- Logged in -->
					<a
						href="/dashboard"
						class="rounded-md px-3 py-2 text-base font-medium block"
						class:bg-gray-900={isActive('/dashboard')}
						class:text-white={isActive('/dashboard')}
						class:text-gray-300={!isActive('/dashboard')}
						class:hover:bg-gray-700={!isActive('/dashboard')}
						class:hover:text-white={!isActive('/dashboard')}
					>
						Dashboard
					</a>
					<a
						href="/find"
						class="rounded-md px-3 py-2 text-base font-medium block"
						class:bg-gray-900={isActive('/find')}
						class:text-white={isActive('/find')}
						class:text-gray-300={!isActive('/find')}
						class:hover:bg-gray-700={!isActive('/find')}
						class:hover:text-white={!isActive('/find')}
					>
						Find Controls
					</a>
					<a
						href="/settings"
						class="rounded-md px-3 py-2 text-base font-medium block"
						class:bg-gray-900={isActive('/settings')}
						class:text-white={isActive('/settings')}
						class:text-gray-300={!isActive('/settings')}
						class:hover:bg-gray-700={!isActive('/settings')}
						class:hover:text-white={!isActive('/settings')}
					>
						Settings
					</a>
					<button
						on:click={handleLogout}
						class="rounded-md px-3 py-2 text-base font-medium block w-full text-left text-gray-300 hover:bg-gray-700 hover:text-white"
					>
						Logout
					</button>
				{/if}
				
				{#if !$isLoading}
					<a
						href="/explore"
						class="rounded-md px-3 py-2 text-base font-medium block"
						class:bg-gray-900={isActive('/explore')}
						class:text-white={isActive('/explore')}
						class:text-gray-300={!isActive('/explore')}
						class:hover:bg-gray-700={!isActive('/explore')}
						class:hover:text-white={!isActive('/explore')}
					>
						Explore
					</a>
					<a
						href="/publication"
						class="rounded-md px-3 py-2 text-base font-medium block"
						class:bg-gray-900={isActive('/publication')}
						class:text-white={isActive('/publication')}
						class:text-gray-300={!isActive('/publication')}
						class:hover:bg-gray-700={!isActive('/publication')}
						class:hover:text-white={!isActive('/publication')}
					>
						Publication
					</a>
				{/if}
			</div>
		</div>
	{/if}
</nav>