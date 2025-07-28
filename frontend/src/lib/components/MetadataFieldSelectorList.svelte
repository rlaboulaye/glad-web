<script lang="ts">
	import { createEventDispatcher } from 'svelte';

	// Props
	export let availableFields: string[] = [];
	export let selectedFields: Set<string> = new Set();
	export let title: string = "";
	export let colorClass: string = "indigo"; // "indigo" for primary, "green" for secondary

	const dispatch = createEventDispatcher();

	// Get tooltip text for metadata fields
	function getFieldTooltip(field: string): string {
		switch (field) {
			case 'phs':
				return 'Unique identifier assigned to a phenotype study by dbGaP';
			case 'country':
				return 'Country where the sample was taken';
			case 'region':
				return 'Region where the sample was taken';
			case 'sex':
				return 'Reported sex of sample';
			case 'ethnicity':
				return 'One of ["Hispanic", "NotHispanic", "NativeAmerican"]';
			case 'self_described':
				return 'True if sample self-reported as "Hispanic"';
			case 'ibd_community':
				return 'Community assignment produced by running Infomap on total pairwise IBD between samples';
			default:
				return '';
		}
	}

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
</div>