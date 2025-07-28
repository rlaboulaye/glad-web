<script lang="ts">
	import { createEventDispatcher } from 'svelte';

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
	
	// PCA-specific features
	export let showColorDots: boolean = false;
	export let getGroupColor: ((label: string) => string) | null = null;

	const dispatch = createEventDispatcher();

	function toggleGroup(groupLabel: string) {
		if (selectedGroups.has(groupLabel)) {
			selectedGroups = new Set([...selectedGroups].filter(g => g !== groupLabel));
		} else {
			selectedGroups = new Set([groupLabel, ...selectedGroups]);
		}
		dispatch('groupsChanged', selectedGroups);
	}

	function togglePrimaryGroup(groupLabel: string) {
		// In cross-grouping mode, primary axis uses main selectedGroups
		if (selectedGroups.has(groupLabel)) {
			selectedGroups = new Set([...selectedGroups].filter(g => g !== groupLabel));
		} else {
			selectedGroups = new Set([groupLabel, ...selectedGroups]);
		}
		dispatch('groupsChanged', selectedGroups);
	}

	function toggleSecondaryGroup(groupLabel: string) {
		if (selectedSecondaryGroups.has(groupLabel)) {
			selectedSecondaryGroups = new Set([...selectedSecondaryGroups].filter(g => g !== groupLabel));
		} else {
			selectedSecondaryGroups = new Set([groupLabel, ...selectedSecondaryGroups]);
		}
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

	function deselectAllXGroups() {
		// In asymmetric mode, X-axis uses main selectedGroups
		selectedGroups = new Set();
		dispatch('groupsChanged', selectedGroups);
	}

	function toggleXGroup(groupLabel: string) {
		// In cross-grouping mode, X-axis uses main selectedGroups
		if (selectedGroups.has(groupLabel)) {
			selectedGroups = new Set([...selectedGroups].filter(g => g !== groupLabel));
		} else {
			selectedGroups = new Set([groupLabel, ...selectedGroups]);
		}
		dispatch('groupsChanged', selectedGroups);
	}

	function toggleYGroup(groupLabel: string) {
		if (selectedSecondaryGroups.has(groupLabel)) {
			selectedSecondaryGroups = new Set([...selectedSecondaryGroups].filter(g => g !== groupLabel));
		} else {
			selectedSecondaryGroups = new Set([groupLabel, ...selectedSecondaryGroups]);
		}
		dispatch('secondaryGroupsChanged', selectedSecondaryGroups);
	}

	function deselectAllYGroups() {
		selectedSecondaryGroups = new Set();
		dispatch('secondaryGroupsChanged', selectedSecondaryGroups);
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
			<!-- Symmetric mode group selection -->
			<div>
				<div class="flex items-center justify-between mb-4">
					<div>
						<p class="text-gray-700 dark:text-gray-300 font-semibold">{title}</p>
						<p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
							{description} ({selectedGroups.size} selected)
						</p>
					</div>
					<div class="flex items-center space-x-3">
						{#if groups.length > 0}
							{#if enableSelectAll}
								<button
									class="px-3 py-1.5 text-sm font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-600 border border-gray-300 dark:border-gray-500 rounded-md hover:bg-gray-200 dark:hover:bg-gray-500 transition-colors duration-200"
									on:click={selectAllGroups}
								>
									Select All
								</button>
							{/if}
							<button
								class="px-3 py-1.5 text-sm font-medium transition-colors duration-200 rounded-md
									{selectedGroups.size > 0 
										? 'text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-600 border border-gray-300 dark:border-gray-500 hover:bg-gray-200 dark:hover:bg-gray-500' 
										: 'text-gray-400 dark:text-gray-500 bg-gray-50 dark:bg-gray-700 border border-gray-200 dark:border-gray-600 cursor-not-allowed'}"
								on:click={deselectAllGroups}
								disabled={selectedGroups.size === 0}
							>
								Deselect All
							</button>
						{/if}
					</div>
				</div>

				{#if groups.length > 0}
					<div class="max-h-60 overflow-y-auto border border-gray-200 dark:border-gray-600 rounded-lg p-4">
						<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-2">
							{#each groups as group}
								<button
									class="px-3 py-2 text-left rounded border text-sm transition-colors duration-200 cursor-pointer select-none
										{selectedGroups.has(group.label)
											? 'bg-indigo-600 border-indigo-600 text-white hover:bg-indigo-700'
											: 'bg-white dark:bg-gray-700 border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-600'}"
									on:click={() => toggleGroup(group.label)}
									disabled={loading}
								>
									{#if showColorDots}
										<div class="flex items-center space-x-2">
											<!-- Color dot -->
											<div 
												class="w-3 h-3 rounded-full border border-gray-300 dark:border-gray-500 flex-shrink-0
													{selectedGroups.has(group.label) ? 'border-white' : ''}"
												style="background-color: {getGroupColor ? getGroupColor(group.label) : '#6b7280'}"
											></div>
											<div class="flex-1 min-w-0">
												<div class="font-medium truncate">{group.label}</div>
												<div class="text-xs opacity-75">{group.size} individuals</div>
											</div>
										</div>
									{:else}
										<div class="font-medium">{group.label}</div>
										<div class="text-xs opacity-75">{group.size} individuals</div>
									{/if}
								</button>
							{/each}
						</div>
					</div>
				{:else if loading}
					<div class="flex items-center justify-center h-32 border border-gray-200 dark:border-gray-600 rounded-lg">
						<div class="text-center">
							<svg class="animate-spin h-6 w-6 text-indigo-600 mx-auto mb-2" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
								<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
								<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 714 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
							</svg>
							<p class="text-sm text-gray-600 dark:text-gray-400">Loading groups...</p>
						</div>
					</div>
				{/if}
			</div>
		{:else}
			<!-- Cross-grouping mode group selection -->
			<div class="space-y-6">
				<!-- X Axis Groups -->
				<div>
					<div class="flex items-center justify-between mb-4">
						<div>
							<p class="text-gray-700 dark:text-gray-300 font-semibold">X Axis Groups:</p>
							<p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
								Choose X-axis groups ({selectedGroups.size} selected)
							</p>
						</div>
						<div class="flex items-center space-x-3">
							{#if selectedGroups.size > 0}
								<button
									class="px-3 py-1.5 text-sm font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-600 border border-gray-300 dark:border-gray-500 rounded-md hover:bg-gray-200 dark:hover:bg-gray-500 transition-colors duration-200"
									on:click={deselectAllXGroups}
								>
									Deselect All
								</button>
							{/if}
						</div>
					</div>

					{#if groups.length > 0}
						<div class="max-h-60 overflow-y-auto border border-gray-200 dark:border-gray-600 rounded-lg p-4">
							<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-2">
								{#each groups as group}
									<button
										class="px-3 py-2 text-left rounded border text-sm transition-colors duration-200 cursor-pointer select-none
											{selectedGroups.has(group.label)
												? 'bg-indigo-600 border-indigo-600 text-white hover:bg-indigo-700'
												: 'bg-white dark:bg-gray-700 border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-600'}"
										on:click={() => toggleXGroup(group.label)}
										disabled={loading}
									>
										<div class="font-medium">{group.label}</div>
										<div class="text-xs opacity-75">{group.size} individuals</div>
									</button>
								{/each}
							</div>
						</div>
					{:else}
						<div class="flex items-center justify-center h-32 border border-gray-200 dark:border-gray-600 rounded-lg">
							<div class="text-center">
								<svg class="animate-spin h-6 w-6 text-indigo-600 mx-auto mb-2" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
									<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
									<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 718-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 714 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
								</svg>
								<p class="text-sm text-gray-600 dark:text-gray-400">Loading X groups...</p>
							</div>
						</div>
					{/if}
				</div>

				<!-- Y Axis Groups -->
				<div>
					<div class="flex items-center justify-between mb-4">
						<div>
							<p class="text-gray-700 dark:text-gray-300 font-semibold">Y Axis Groups:</p>
							<p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
								Choose Y-axis groups ({selectedSecondaryGroups.size} selected)
							</p>
						</div>
						<div class="flex items-center space-x-3">
							{#if selectedSecondaryGroups.size > 0}
								<button
									class="px-3 py-1.5 text-sm font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-600 border border-gray-300 dark:border-gray-500 rounded-md hover:bg-gray-200 dark:hover:bg-gray-500 transition-colors duration-200"
									on:click={deselectAllYGroups}
								>
									Deselect All
								</button>
							{/if}
						</div>
					</div>

					{#if secondaryGroups.length > 0}
						<div class="max-h-60 overflow-y-auto border border-gray-200 dark:border-gray-600 rounded-lg p-4">
							<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-2">
								{#each secondaryGroups as group}
									<button
										class="px-3 py-2 text-left rounded border text-sm transition-colors duration-200 cursor-pointer select-none
											{selectedSecondaryGroups.has(group.label)
												? 'bg-green-600 border-green-600 text-white hover:bg-green-700'
												: 'bg-white dark:bg-gray-700 border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-600'}"
										on:click={() => toggleYGroup(group.label)}
										disabled={loading}
									>
										<div class="font-medium">{group.label}</div>
										<div class="text-xs opacity-75">{group.size} individuals</div>
									</button>
								{/each}
							</div>
						</div>
					{:else}
						<div class="flex items-center justify-center h-32 border border-gray-200 dark:border-gray-600 rounded-lg">
							<div class="text-center">
								<svg class="animate-spin h-6 w-6 text-indigo-600 mx-auto mb-2" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
									<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
									<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 718-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 714 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
								</svg>
								<p class="text-sm text-gray-600 dark:text-gray-400">Loading Y groups...</p>
							</div>
						</div>
					{/if}
				</div>
			</div>
		{/if}
	</div>
</div>