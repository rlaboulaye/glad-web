<script lang="ts">
	import { createEventDispatcher } from 'svelte';

	// Props
	export let availableFields: string[] = [];
	export let selectedFields: Set<string> = new Set();
	export let xFields: Set<string> = new Set();
	export let yFields: Set<string> = new Set(); 
	export let asymmetricMode: boolean = false;
	export let proposeAsymmetric: boolean = false;
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

	function toggleXField(field: string) {
		if (xFields.has(field)) {
			xFields = new Set([...xFields].filter(f => f !== field));
		} else {
			xFields = new Set([field, ...xFields]);
		}
		dispatch('xFieldsChanged', xFields);
	}

	function toggleYField(field: string) {
		if (yFields.has(field)) {
			yFields = new Set([...yFields].filter(f => f !== field));
		} else {
			yFields = new Set([field, ...yFields]);
		}
		dispatch('yFieldsChanged', yFields);
	}

	function toggleAsymmetricMode() {
		asymmetricMode = !asymmetricMode;
		dispatch('asymmetricModeToggled', asymmetricMode);
	}
</script>

<div class="bg-white dark:bg-gray-800 shadow-md rounded-lg p-6 mb-6">
	<div class="space-y-4">
		<div class="flex items-center justify-between">
			<p class="text-gray-700 dark:text-gray-300 font-semibold">{title}</p>
			<!-- Always reserve space for the button to prevent layout shift -->
			<div class="h-10 flex items-center">
				{#if proposeAsymmetric}
					<button
						class="px-4 py-2 text-sm font-medium rounded-md border transition-colors duration-200 bg-gray-100 dark:bg-gray-600 border-gray-300 dark:border-gray-500 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-500"
						on:click={toggleAsymmetricMode}
					>
						{asymmetricMode ? '🔄 Switch to Symmetric' : '🔄 Switch to Asymmetric'}
					</button>
				{:else}
					<!-- Invisible placeholder to maintain consistent spacing -->
					<div class="px-4 py-2 text-sm font-medium opacity-0 pointer-events-none">
						🔄 Switch to Asymmetric
					</div>
				{/if}
			</div>
		</div>
		
		{#if !asymmetricMode}
			<!-- Symmetric mode field selector -->
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
			<!-- Asymmetric mode X/Y axis selectors -->
			<div class="space-y-4">
				<!-- X Axis -->
				<div>
					<p class="text-gray-700 dark:text-gray-300 mb-2 font-medium">X Axis:</p>
					<div class="flex flex-wrap gap-2">
						{#each availableFields as field}
							<button
								class="px-4 py-2 rounded-full border text-sm font-medium transition-colors duration-200 cursor-pointer select-none
									{xFields.has(field)
										? 'bg-indigo-600 border-indigo-600 text-white hover:bg-indigo-700'
										: 'bg-white dark:bg-gray-700 border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-600'}"
								title={getFieldTooltip(field)}
								on:click={() => toggleXField(field)}
							>
								{field}
							</button>
						{/each}
					</div>
				</div>
				
				<!-- Y Axis -->
				<div>
					<p class="text-gray-700 dark:text-gray-300 mb-2 font-medium">Y Axis:</p>
					<div class="flex flex-wrap gap-2">
						{#each availableFields as field}
							<button
								class="px-4 py-2 rounded-full border text-sm font-medium transition-colors duration-200 cursor-pointer select-none
									{yFields.has(field)
										? 'bg-green-600 border-green-600 text-white hover:bg-green-700'
										: 'bg-white dark:bg-gray-700 border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-600'}"
								title={getFieldTooltip(field)}
								on:click={() => toggleYField(field)}
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