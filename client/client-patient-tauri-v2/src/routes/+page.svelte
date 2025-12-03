<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';

	async function test() {
		await invoke('get_keys_entry');
	}
</script>

<button onclick={test} class="border p-4">Test</button>
