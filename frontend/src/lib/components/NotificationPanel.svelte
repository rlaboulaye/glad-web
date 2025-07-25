<script>
	import { notifications, markNotificationsAsRead, markAllNotificationsAsRead, closeNotificationPanel } from '$lib/notifications.js';
	import { addToast } from '$lib/toast.ts';
	import { goto } from '$app/navigation';

	export let isOpen = false;

	// Format timestamp for display
	function formatTimestamp(timestamp) {
		const date = new Date(timestamp);
		const now = new Date();
		const diffMs = now - date;
		const diffMinutes = Math.floor(diffMs / (1000 * 60));
		const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
		const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

		if (diffMinutes < 1) return 'Just now';
		if (diffMinutes < 60) return `${diffMinutes}m ago`;
		if (diffHours < 24) return `${diffHours}h ago`;
		if (diffDays < 7) return `${diffDays}d ago`;
		
		return date.toLocaleDateString();
	}

	// Mark individual notification as read
	async function handleMarkAsRead(notificationId) {
		try {
			// Check if this is the last unread notification
			const unreadNotifications = $notifications.filter(n => !n.is_read);
			const isLastNotification = unreadNotifications.length === 1;
			
			await markNotificationsAsRead([notificationId]);
			
			// Close panel if this was the last notification
			if (isLastNotification) {
				closeNotificationPanel();
			}
		} catch (error) {
			addToast('error', 'Failed to mark notification as read');
		}
	}

	// Mark all notifications as read
	async function handleMarkAllAsRead() {
		try {
			await markAllNotificationsAsRead();
			addToast('success', 'All notifications marked as read');
		} catch (error) {
			addToast('error', 'Failed to mark all notifications as read');
		}
	}

	// Handle clicking outside to close panel
	function handleOutsideClick(event) {
		if (event.target.closest('.notification-panel') === null) {
			closeNotificationPanel();
		}
	}
</script>

<svelte:window on:click={handleOutsideClick} />

{#if isOpen}
	<div class="notification-panel absolute right-0 top-12 z-50 w-96 max-w-sm bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg">
		<!-- Header -->
		<div class="px-4 py-3 border-b border-gray-200 dark:border-gray-700 flex justify-between items-center">
			<h3 class="text-lg font-medium text-gray-900 dark:text-white">Notifications</h3>
			{#if $notifications.filter(n => !n.is_read).length > 0}
				<button 
					on:click|stopPropagation={handleMarkAllAsRead}
					class="text-sm text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-300"
				>
					Mark all read
				</button>
			{/if}
		</div>

		<!-- Notification list -->
		<div class="max-h-96 overflow-y-auto">
			{#if $notifications.filter(n => !n.is_read).length === 0}
				<div class="px-4 py-8 text-center text-gray-500 dark:text-gray-400">
					<svg class="mx-auto h-12 w-12 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17h5l-5 5l-5-5h5v-12" />
					</svg>
					<p class="mt-2">No notifications yet</p>
				</div>
			{:else}
				{#each $notifications.filter(n => !n.is_read) as notification (notification.notification_id)}
					<button 
						type="button"
						class="block w-full text-left px-4 py-3 border-b border-gray-100 dark:border-gray-700 last:border-b-0 hover:bg-gray-50 dark:hover:bg-gray-700 bg-blue-50 dark:bg-blue-900/20"
						on:click={async () => {
							closeNotificationPanel();
							// Mark as read when clicked
							await handleMarkAsRead(notification.notification_id);
							// Navigate to query page
							goto(`/dashboard/query/${notification.query_id}`);
						}}
						aria-label="View query {notification.query_id} details"
					>
						<div class="flex justify-between items-start">
							<div class="flex-1 min-w-0">
								<div class="flex items-center space-x-2">
									<p class="text-sm font-medium text-gray-900 dark:text-white">
										{notification.title}
									</p>
									<span class="inline-block w-2 h-2 bg-blue-600 rounded-full"></span>
								</div>
								<p class="mt-1 text-sm text-gray-600 dark:text-gray-300">
									{notification.message}
								</p>
								<p class="mt-1 text-xs text-gray-400 dark:text-gray-500">
									{formatTimestamp(notification.created_at)}
								</p>
							</div>
							<span 
								role="button"
								tabindex="0"
								on:click|stopPropagation|preventDefault={() => handleMarkAsRead(notification.notification_id)}
								on:keydown={(e) => e.key === 'Enter' && handleMarkAsRead(notification.notification_id)}
								class="ml-3 text-xs text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-300 cursor-pointer"
								aria-label="Mark notification as read"
							>
								Mark read
							</span>
						</div>
					</button>
				{/each}
			{/if}
		</div>

	</div>
{/if}