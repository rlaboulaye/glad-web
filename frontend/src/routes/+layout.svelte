<script>
	import '../app.css';
	import Toast from '$lib/components/Toast.svelte';
	import Navigation from '$lib/components/Navigation.svelte';
	import PageLogos from '$lib/components/PageLogos.svelte';
</script>

<Toast />
<Navigation />
<div class="flex flex-col bg-gray-100 dark:bg-gray-900" style="min-height: calc(100vh - 4rem);">
	<main class="flex-1">
		<slot />
	</main>
	<PageLogos />
</div>