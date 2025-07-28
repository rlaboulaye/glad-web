<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import MetadataFieldSelectorList from './MetadataFieldSelectorList.svelte';

	// Props - purely controlled component
	export let availableFields: string[] = [];
	export let selectedFields: Set<string> = new Set();
	export let secondaryFields: Set<string> = new Set(); 
	export let crossGroupingMode: boolean = false;
	export let proposeCrossGrouping: boolean = false;
	export let title: string = "Group by Metadata:";

	const dispatch = createEventDispatcher();

	function handlePrimaryFieldsChanged(event) {
		selectedFields = event.detail;
		dispatch('fieldsChanged', selectedFields);
	}

	function handleSecondaryFieldsChanged(event) {
		secondaryFields = event.detail;
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
			<MetadataFieldSelectorList 
				{availableFields}
				bind:selectedFields
				colorClass="indigo"
				on:fieldsChanged={handlePrimaryFieldsChanged}
			/>
		{:else}
			<!-- Cross-grouping mode primary/secondary axis selectors -->
			<div class="space-y-4">
				<!-- Primary Axis -->
				<MetadataFieldSelectorList 
					{availableFields}
					bind:selectedFields
					title="Primary Axis:"
					colorClass="indigo"
					on:fieldsChanged={handlePrimaryFieldsChanged}
				/>
				
				<!-- Secondary Axis -->
				<MetadataFieldSelectorList 
					{availableFields}
					bind:selectedFields={secondaryFields}
					title="Secondary Axis:"
					colorClass="green"
					on:fieldsChanged={handleSecondaryFieldsChanged}
				/>
			</div>
		{/if}
	</div>
</div>