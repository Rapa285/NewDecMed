import { invalidateAll } from '$app/navigation';
import { addHospitalSchema } from '$lib/schema';
import type { AddHospitalSchema, SuccessResponse } from '$lib/types';
import { tryCatchAsVal, waitMs } from '$lib/utils';
import { invoke } from '@tauri-apps/api/core';
import { toast } from 'svelte-sonner';
import { type Infer, type SuperValidated, type SuperForm, superForm } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';

type ConstructorProps = {
	addHospitalForm: SuperValidated<Infer<AddHospitalSchema>>;
};

export class HomeState {
	isAddHospitalDialogOpen = $state(false);
	addHospitalFormMeta: SuperForm<Infer<AddHospitalSchema>>;
	isLoadingUpdateActivationKey = $state(false);

	constructor({ addHospitalForm }: ConstructorProps) {
		this.addHospitalFormMeta = superForm(addHospitalForm, {
			validators: zod(addHospitalSchema),
			dataType: 'json',
			SPA: true,
			delayMs: 100,

			onUpdate: async ({ result, form, cancel }) => {
				if (result.type === 'success') {
					const resInvokeCreateActivationKey = await tryCatchAsVal(async () => {
						return await invoke('create_activation_key', {
							payload: {
								hospitalId: form.data.hospitalId,
								hospitalName: form.data.hospitalName
							}
						});
					});

					if (!resInvokeCreateActivationKey.success) {
						toast.error(resInvokeCreateActivationKey.error);
						cancel();
						return;
					}

					await waitMs(2000);
					this.isAddHospitalDialogOpen = false;

					toast.success('Hospital successfully registered');
				}
			}
		});
	}

	updateActivationKey = async ({ hospitalAdminCid }: { hospitalAdminCid: string }) => {
		const resInvokeUpdateActivationKey = await tryCatchAsVal(async () => {
			return (await invoke('update_activation_key', {
				payload: { hospitalAdminCid }
			})) as SuccessResponse<null>;
		});

		if (!resInvokeUpdateActivationKey.success) {
			this.isLoadingUpdateActivationKey = false;
			toast.error(resInvokeUpdateActivationKey.error);
			return;
		}

		await waitMs(2000);
		invalidateAll();
		this.isLoadingUpdateActivationKey = false;
		toast.success('Activation key updated successfully');
	};

	openAddHospitalDialog = () => {
		this.isAddHospitalDialogOpen = true;
	};
	closeAddHospitalDialog = () => {
		this.isAddHospitalDialogOpen = false;
	};
}
