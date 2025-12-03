<script lang="ts">
	import { getAuthContext } from '../../(context)/auth-context.svelte';
	import Admin from './admin.svelte';
	import Personnel from './personnel.svelte';

	let { data } = $props();
	const authContext = getAuthContext();
</script>

<div class="flex flex-col max-w-md w-full">
	<h2 class="font-montserrat text-xl font-medium my-4">Complete Your Profile</h2>
	{#if authContext.role === 'Admin'}
		<Admin completeProfileAdminFormData={data.completeProfileAdminForm} />
	{:else}
		<Personnel completeProfilePersonnelFormData={data.completeProfilePersonnelForm} />
	{/if}
</div>
