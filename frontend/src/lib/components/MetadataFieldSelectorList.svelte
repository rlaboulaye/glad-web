<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import { getFieldTooltip } from '$lib/data/fieldTooltips';

	// Props
	export let availableFields: string[] = [];
	export let selectedFields: Set<string> = new Set();
	export let title: string = "";
	export let colorClass: string = "indigo"; // "indigo" for primary, "green" for secondary
	export let showDescriptions: boolean = false; // New prop for layout toggle

	const dispatch = createEventDispatcher();

	function toggleField(field: string) {
		if (selectedFields.has(field)) {
			selectedFields.delete(field);
		} else {
			selectedFields.add(field);
		}
		selectedFields = selectedFields; // Trigger reactivity
		dispatch('fieldsChanged', selectedFields);
	}

	// Generate color classes based on colorClass prop
	$: selectedClasses = colorClass === 'green' 
		? 'bg-green-600 border-green-600 text-white hover:bg-green-700'
		: 'bg-indigo-600 border-indigo-600 text-white hover:bg-indigo-700';
	
	$: unselectedClasses = 'bg-white dark:bg-gray-700 border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-600';
</script>

<div>
	{#if title}
		<p class="text-gray-700 dark:text-gray-300 mb-2 font-medium">{title}</p>
	{/if}
	
	{#if showDescriptions}
		<!-- Vertical layout with aligned descriptions -->
		<div class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-3 items-start">
			{#each availableFields as field}
				<button
					class="px-4 py-2 rounded-full border text-sm font-medium transition-colors duration-200 cursor-pointer select-none whitespace-nowrap
						{selectedFields.has(field) ? selectedClasses : unselectedClasses}"
					on:click={() => toggleField(field)}
				>
					{field}
				</button>
				<div class="pt-2 text-sm text-gray-600 dark:text-gray-400">
					{getFieldTooltip(field)}
				</div>
			{/each}
		</div>
	{:else}
		<!-- Horizontal layout with tooltips -->
		<div class="flex flex-wrap gap-2">
			{#each availableFields as field}
				<button
					class="px-4 py-2 rounded-full border text-sm font-medium transition-colors duration-200 cursor-pointer select-none
						{selectedFields.has(field) ? selectedClasses : unselectedClasses}"
					title={getFieldTooltip(field)}
					on:click={() => toggleField(field)}
				>
					{field}
				</button>
			{/each}
		</div>
	{/if}
</div>