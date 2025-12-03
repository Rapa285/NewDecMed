import { completeProfileAdminSchema } from '$lib/schema';
import type { CompleteProfileAdminSchema, SuccessResponse } from '$lib/types';
import { tryCatchAsVal } from '$lib/utils';
import { invoke } from '@tauri-apps/api/core';
import { toast } from 'svelte-sonner';
import { superForm, type Infer, type SuperForm, type SuperValidated } from 'sveltekit-superforms';
import { zodClient } from 'sveltekit-superforms/adapters';

type Constructor = {
	completeProfileAdminForm: SuperValidated<Infer<CompleteProfileAdminSchema>>;
};

export class CompleteProfileAdminState {
	completeProfileAdminFormMeta: SuperForm<Infer<CompleteProfileAdminSchema>>;

	constructor({ completeProfileAdminForm }: Constructor) {
		this.completeProfileAdminFormMeta = superForm(completeProfileAdminForm, {
			validators: zodClient(completeProfileAdminSchema),
			dataType: 'json',
			SPA: true,
			onUpdate: async ({ result, form, cancel }) => {
				if (result.type === 'success') {
					const resultInvokeUpdateProfile = await tryCatchAsVal(async () => {
						return (await invoke('update_profile', {
							data: {
								name: form.data.name
							}
						})) as SuccessResponse<null>;
					});

					if (!resultInvokeUpdateProfile.success) {
						cancel();
						toast.error(resultInvokeUpdateProfile.error);
						return;
					}

					toast.success('Profile updated successfully');
				}
			}
		});
	}
}
