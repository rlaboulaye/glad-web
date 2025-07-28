<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import GroupSelectorList from './GroupSelectorList.svelte';

	// Props
	export let groups: Array<{label: string, size: number}> = [];
	export let selectedGroups: Set<string> = new Set();
	export let loading: boolean = false;
	export let title: string = "Group Selection:";
	export let description: string = "Choose groups to display";
	export let enableSelectAll: boolean = false;
	
	// Cross-grouping mode support
	export let crossGroupingMode: boolean = false;
	export let secondaryGroups: Array<{label: string, size: number}> = [];
	export let selectedSecondaryGroups: Set<string> = new Set();
	export let primaryFieldsLabel: string = "Primary Axis";
	export let secondaryFieldsLabel: string = "Secondary Axis";
	
	// PCA-specific features
	export let showColorDots: boolean = false;
	export let getGroupColor: ((label: string) => string) | null = null;

	const dispatch = createEventDispatcher();

	function handlePrimaryGroupsChanged(event) {
		selectedGroups = event.detail;
		dispatch('groupsChanged', selectedGroups);
	}

	function handleSecondaryGroupsChanged(event) {
		selectedSecondaryGroups = event.detail;
		dispatch('secondaryGroupsChanged', selectedSecondaryGroups);
	}

	function selectAllGroups() {
		selectedGroups = new Set(groups.map(g => g.label));
		dispatch('selectAll');
		dispatch('groupsChanged', selectedGroups);
	}

	function deselectAllGroups() {
		selectedGroups = new Set();
		dispatch('deselectAll');
		dispatch('groupsChanged', selectedGroups);
	}
</script>

<div class="bg-white dark:bg-gray-800 shadow-md rounded-lg p-6 mb-6">
	<div class="space-y-6">
		{#if (!crossGroupingMode && groups.length === 0) || (crossGroupingMode && (groups.length === 0 || secondaryGroups.length === 0))}
			<div class="flex items-center justify-center h-32 border border-gray-200 dark:border-gray-600 rounded-lg">
				<div class="text-center">
					<div class="text-3xl mb-2">🏷️</div>
					<h3 class="text-lg font-semibold text-gray-700 dark:text-gray-300 mb-2">Select Metadata Fields</h3>
					<p class="text-gray-600 dark:text-gray-400">
						Choose metadata fields above to group the data
					</p>
				</div>
			</div>
		{:else if !crossGroupingMode}
			<!-- Basic mode group selection -->
			<GroupSelectorList 
				{groups}
				bind:selectedGroups
				{title}
				{description}
				{loading}
				{showColorDots}
				{getGroupColor}
				colorClass="indigo"
				showSelectAll={enableSelectAll}
				on:groupsChanged={handlePrimaryGroupsChanged}
				on:selectAll={selectAllGroups}
				on:deselectAll={deselectAllGroups}
			/>
		{:else}
			<!-- Cross-grouping mode group selection -->
			<div class="space-y-6">
				<!-- Primary Groups -->
				<GroupSelectorList 
					{groups}
					bind:selectedGroups
					title="{primaryFieldsLabel} Groups:"
					description="Choose {primaryFieldsLabel.toLowerCase()} groups"
					{loading}
					showColorDots={false}
					colorClass="indigo"
					showSelectAll={false}
					on:groupsChanged={handlePrimaryGroupsChanged}
				/>

				<!-- Secondary Groups -->
				<GroupSelectorList 
					groups={secondaryGroups}
					bind:selectedGroups={selectedSecondaryGroups}
					title="{secondaryFieldsLabel} Groups:"
					description="Choose {secondaryFieldsLabel.toLowerCase()} groups"
					{loading}
					showColorDots={false}
					colorClass="green"
					showSelectAll={false}
					on:groupsChanged={handleSecondaryGroupsChanged}
				/>
			</div>
		{/if}
	</div>
</div>