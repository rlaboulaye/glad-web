<script>
	import '../app.css';
	import Toast from '$lib/components/Toast.svelte';
	import Navigation from '$lib/components/Navigation.svelte';
</script>

<Toast />
<Navigation />
<main>
	<slot />
</main>