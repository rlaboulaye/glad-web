<script lang="ts">
	import { createEventDispatcher } from 'svelte';

	// Props - purely controlled component
	export let availableFields: string[] = [];
	export let selectedFields: Set<string> = new Set();
	export let secondaryFields: Set<string> = new Set(); 
	export let crossGroupingMode: boolean = false;
	export let proposeCrossGrouping: boolean = false;
	export let title: string = "Group by Metadata:";

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

	function toggleSymmetricField(field: string) {
		if (selectedFields.has(field)) {
			selectedFields.delete(field);
		} else {
			selectedFields.add(field);
		}
		selectedFields = selectedFields; // Trigger reactivity
		dispatch('fieldsChanged', selectedFields);
	}

	function togglePrimaryField(field: string) {
		// In cross-grouping mode, primary axis uses main selectedFields
		if (selectedFields.has(field)) {
			selectedFields.delete(field);
		} else {
			selectedFields.add(field);
		}
		selectedFields = selectedFields; // Trigger reactivity
		dispatch('fieldsChanged', selectedFields);
	}

	function toggleSecondaryField(field: string) {
		if (secondaryFields.has(field)) {
			secondaryFields = new Set([...secondaryFields].filter(f => f !== field));
		} else {
			secondaryFields = new Set([field, ...secondaryFields]);
		}
		dispatch('secondaryFieldsChanged', secondaryFields);
	}

	function toggleCrossGroupingMode() {
		crossGroupingMode = !crossGroupingMode;
		dispatch('crossGroupingModeToggled', crossGroupingMode);
	}
</script>

<div class="bg-white dark:bg-gray-800 shadow-md rounded-lg p-6 mb-6">
	<div class="space-y-4">
		<div class="flex items-center justify-between">
			<p class="text-gray-700 dark:text-gray-300 font-semibold">{title}</p>
			<!-- Always reserve space for the button to prevent layout shift -->
			<div class="h-10 flex items-center">
				{#if proposeCrossGrouping}
					<button
						class="px-4 py-2 text-sm font-medium rounded-md border transition-colors duration-200 bg-gray-100 dark:bg-gray-600 border-gray-300 dark:border-gray-500 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-500"
						on:click={toggleCrossGroupingMode}
					>
						{crossGroupingMode ? '🔄 Switch to Basic' : '🔄 Switch to Cross-Grouping'}
					</button>
				{:else}
					<!-- Invisible placeholder to maintain consistent spacing -->
					<div class="px-4 py-2 text-sm font-medium opacity-0 pointer-events-none">
						🔄 Switch to Cross-Grouping
					</div>
				{/if}
			</div>
		</div>
		
		{#if !crossGroupingMode}
			<!-- Basic mode field selector -->
			<div class="flex flex-wrap gap-2">
				{#each availableFields as field}
					<button
						class="px-4 py-2 rounded-full border text-sm font-medium transition-colors duration-200 cursor-pointer select-none
							{selectedFields.has(field)
								? 'bg-indigo-600 border-indigo-600 text-white hover:bg-indigo-700'
								: 'bg-white dark:bg-gray-700 border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-600'}"
						title={getFieldTooltip(field)}
						on:click={() => toggleSymmetricField(field)}
					>
						{field}
					</button>
				{/each}
			</div>
		{:else}
			<!-- Cross-grouping mode primary/secondary axis selectors -->
			<div class="space-y-4">
				<!-- Primary Axis -->
				<div>
					<p class="text-gray-700 dark:text-gray-300 mb-2 font-medium">Primary Axis:</p>
					<div class="flex flex-wrap gap-2">
						{#each availableFields as field}
							<button
								class="px-4 py-2 rounded-full border text-sm font-medium transition-colors duration-200 cursor-pointer select-none
									{selectedFields.has(field)
										? 'bg-indigo-600 border-indigo-600 text-white hover:bg-indigo-700'
										: 'bg-white dark:bg-gray-700 border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-600'}"
								title={getFieldTooltip(field)}
								on:click={() => togglePrimaryField(field)}
							>
								{field}
							</button>
						{/each}
					</div>
				</div>
				
				<!-- Secondary Axis -->
				<div>
					<p class="text-gray-700 dark:text-gray-300 mb-2 font-medium">Secondary Axis:</p>
					<div class="flex flex-wrap gap-2">
						{#each availableFields as field}
							<button
								class="px-4 py-2 rounded-full border text-sm font-medium transition-colors duration-200 cursor-pointer select-none
									{secondaryFields.has(field)
										? 'bg-green-600 border-green-600 text-white hover:bg-green-700'
										: 'bg-white dark:bg-gray-700 border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-600'}"
								title={getFieldTooltip(field)}
								on:click={() => toggleSecondaryField(field)}
							>
								{field}
							</button>
						{/each}
					</div>
				</div>
			</div>
		{/if}
	</div>
</div>