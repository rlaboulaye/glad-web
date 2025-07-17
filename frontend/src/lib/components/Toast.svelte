<script>
	import { toasts, removeToast } from '$lib/toast.js';
	import { fly } from 'svelte/transition';
</script>

<div class="fixed top-4 right-4 z-50 space-y-2">
	{#each $toasts as toast (toast.id)}
		<div
			transition:fly={{ x: 300, duration: 300 }}
			class="flex items-center p-4 rounded-lg shadow-lg max-w-sm"
			class:bg-green-500={toast.type === 'success'}
			class:bg-red-500={toast.type === 'error'}
			class:bg-blue-500={toast.type === 'info'}
			class:text-white={true}
		>
			<div class="flex-1">
				{toast.message}
			</div>
			<button
				on:click={() => removeToast(toast.id)}
				class="ml-2 text-white hover:text-gray-200"
				aria-label="Close"
			>
				×
			</button>
		</div>
	{/each}
</div>