<script lang="ts">
	import { createEventDispatcher } from 'svelte';

	// Props
	export let groups: Array<{label: string, size: number}> = [];
	export let selectedGroups: Set<string> = new Set();
	export let title: string = "";
	export let description: string = "";
	export let loading: boolean = false;
	export let showColorDots: boolean = false;
	export let getGroupColor: ((label: string) => string) | null = null;
	export let colorClass: string = "indigo"; // "indigo" for primary/X-axis, "green" for secondary/Y-axis
	export let showSelectAll: boolean = false;

	const dispatch = createEventDispatcher();

	function toggleGroup(groupLabel: string) {
		if (selectedGroups.has(groupLabel)) {
			selectedGroups = new Set([...selectedGroups].filter(g => g !== groupLabel));
		} else {
			selectedGroups = new Set([groupLabel, ...selectedGroups]);
		}
		dispatch('groupsChanged', selectedGroups);
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

	// Generate color classes based on colorClass prop
	$: selectedClasses = colorClass === 'green' 
		? 'bg-green-600 border-green-600 text-white hover:bg-green-700'
		: 'bg-indigo-600 border-indigo-600 text-white hover:bg-indigo-700';
	
	$: unselectedClasses = 'bg-white dark:bg-gray-700 border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-600';
</script>

<div>
	<!-- Header with title, description, and deselect button -->
	<div class="flex items-center justify-between mb-4">
		<div>
			<p class="text-gray-700 dark:text-gray-300 font-semibold">{title}</p>
			<p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
				{description} ({selectedGroups.size} selected)
			</p>
		</div>
		<div class="flex items-center space-x-3">
			{#if groups.length > 0}
				{#if showSelectAll}
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

	<!-- Groups grid -->
	{#if groups.length > 0}
		<div class="max-h-60 overflow-y-auto border border-gray-200 dark:border-gray-600 rounded-lg p-4">
			<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-2">
				{#each groups as group}
					<button
						class="px-3 py-2 text-left rounded border text-sm transition-colors duration-200 cursor-pointer select-none
							{selectedGroups.has(group.label) ? selectedClasses : unselectedClasses}"
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