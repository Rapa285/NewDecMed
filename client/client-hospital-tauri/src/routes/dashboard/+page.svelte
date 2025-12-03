<script lang="ts">
	import HomeAdmin from './home-admin.svelte';
	import HomeAdministrative from './home-administrative.svelte';
	import HomeMedical from './home-medical.svelte';
	import {
		ADMIN_ROLE,
		ADMINISTRATIVE_PERSONNEL_ROLE,
		MEDICAL_PERSONNEL_ROLE
	} from '$lib/constants';

	let { data } = $props();

	console.log(data.role);
</script>

{#if data.role === ADMIN_ROLE}
	<HomeAdmin addPersonnelFormData={data.addPersonnelForm!} />
{:else if data.role === MEDICAL_PERSONNEL_ROLE}
	<HomeMedical />
{:else if data.role === ADMINISTRATIVE_PERSONNEL_ROLE}
	<HomeAdministrative />
{/if}
