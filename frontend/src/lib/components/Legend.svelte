<script lang="ts">
	export let legendData: Array<{label: string, color: string, size: number}> = [];
	export let maxHeight = "400px";
	export let showCounts = true;
	export let title = "Legend";
</script>

{#if legendData.length > 0}
	<div class="bg-white dark:bg-gray-800 shadow-md rounded-lg p-6">
		<div class="mb-4">
			<h3 class="text-lg font-semibold text-gray-700 dark:text-gray-300">{title}</h3>
			<p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
				Colors for currently visible groups ({legendData.length} groups)
			</p>
		</div>

		<div 
			class="overflow-y-auto border border-gray-200 dark:border-gray-600 rounded-lg p-4"
			style="max-height: {maxHeight}"
		>
			<div class="space-y-2">
				{#each legendData as group}
					<div class="flex items-center space-x-3 py-1">
						<!-- Color swatch -->
						<div 
							class="w-4 h-4 rounded-full border border-gray-300 dark:border-gray-500 flex-shrink-0"
							style="background-color: {group.color}"
						></div>
						
						<!-- Group label and count -->
						<div class="flex-1 min-w-0">
							<div class="text-sm text-gray-700 dark:text-gray-300 truncate">
								{group.label}
								{#if showCounts}
									<span class="text-gray-500 dark:text-gray-400">
										({group.size})
									</span>
								{/if}
							</div>
						</div>
					</div>
				{/each}
			</div>
		</div>
	</div>
{/if}