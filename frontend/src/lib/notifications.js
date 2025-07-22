import { writable } from 'svelte/store';

export const notifications = writable([]);
export const unreadCount = writable(0);
export const isNotificationPanelOpen = writable(false);

// API functions
export async function fetchNotifications() {
    try {
        const response = await fetch('/api/notifications', {
            credentials: 'include'
        });
        
        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }
        
        const data = await response.json();
        notifications.set(data.notifications);
        unreadCount.set(data.unread_count);
        return data;
    } catch (error) {
        console.error('Failed to fetch notifications:', error);
        throw error;
    }
}

export async function fetchUnreadCount() {
    try {
        const response = await fetch('/api/notifications/unread-count', {
            credentials: 'include'
        });
        
        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }
        
        const data = await response.json();
        unreadCount.set(data.unread_count);
        return data.unread_count;
    } catch (error) {
        console.error('Failed to fetch unread count:', error);
        throw error;
    }
}

export async function markNotificationsAsRead(notificationIds) {
    try {
        const response = await fetch('/api/notifications/mark-read', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            credentials: 'include',
            body: JSON.stringify({ notification_ids: notificationIds })
        });
        
        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }
        
        // Refresh notifications after marking as read
        await fetchNotifications();
        return response.json();
    } catch (error) {
        console.error('Failed to mark notifications as read:', error);
        throw error;
    }
}

export async function markAllNotificationsAsRead() {
    try {
        const response = await fetch('/api/notifications/mark-all-read', {
            method: 'POST',
            credentials: 'include'
        });
        
        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }
        
        // Refresh notifications after marking all as read
        await fetchNotifications();
        return response.json();
    } catch (error) {
        console.error('Failed to mark all notifications as read:', error);
        throw error;
    }
}

// Helper functions
export function toggleNotificationPanel() {
    isNotificationPanelOpen.update(open => !open);
}

export function closeNotificationPanel() {
    isNotificationPanelOpen.set(false);
}