import { completeProfilePersonnelSchema } from '$lib/schema';
import type { CompleteProfilePersonnelSchema, SuccessResponse } from '$lib/types';
import { tryCatchAsVal } from '$lib/utils';
import { invoke } from '@tauri-apps/api/core';
import { toast } from 'svelte-sonner';
import { superForm, type Infer, type SuperForm, type SuperValidated } from 'sveltekit-superforms';
import { zodClient } from 'sveltekit-superforms/adapters';

type Constructor = {
	completeProfilePersonnelForm: SuperValidated<Infer<CompleteProfilePersonnelSchema>>;
};

export class CompleteProfilePersonnelState {
	completeProfilePersonnelFormMeta: SuperForm<Infer<CompleteProfilePersonnelSchema>>;

	constructor({ completeProfilePersonnelForm }: Constructor) {
		this.completeProfilePersonnelFormMeta = superForm(completeProfilePersonnelForm, {
			validators: zodClient(completeProfilePersonnelSchema),
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
