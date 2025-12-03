import { ADMINISTRATIVE_PERSONNEL_ROLE, MEDICAL_PERSONNEL_ROLE } from '$lib/constants';
import { addPersonnelSchemas } from '$lib/schema';
import type {
	Account,
	AddPersonnelSchemaStep2,
	HospitalPersonnel,
	InvokeHospitalAdminAddActivationKeyResponse,
	SuccessResponse
} from '$lib/types';
import { tryCatchAsVal } from '$lib/utils';
import { invoke } from '@tauri-apps/api/core';
import { toast } from 'svelte-sonner';
import { superForm, type Infer, type SuperForm, type SuperValidated } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';

type Constructor = {
	addPersonnelForm: SuperValidated<Infer<AddPersonnelSchemaStep2>>;
};

export class AdminHomeState {
	currentStep = $state(1);
	addPersonnelDialogOpen = $state(false);
	askPin = $state(false);
	something: Infer<AddPersonnelSchemaStep2> | undefined = undefined;
	addPersonnelFormMeta: SuperForm<Infer<AddPersonnelSchemaStep2>>;
	accounts: Account[] = [
		{
			id: 'ADM-111111',
			name: 'Administrative 1',
			role: ADMINISTRATIVE_PERSONNEL_ROLE
		}
	];
	roles = [
		{
			value: MEDICAL_PERSONNEL_ROLE,
			label: MEDICAL_PERSONNEL_ROLE
		},
		{
			value: ADMINISTRATIVE_PERSONNEL_ROLE,
			label: ADMINISTRATIVE_PERSONNEL_ROLE
		}
	];

	constructor({ addPersonnelForm }: Constructor) {
		$effect(() => {
			this.addPersonnelFormMeta.options.validators = zod(addPersonnelSchemas[this.currentStep - 1]);
		});

		this.addPersonnelFormMeta = superForm(addPersonnelForm, {
			validators: false,
			dataType: 'json',
			SPA: true,
			invalidateAll: false,
			onSubmit: async ({ cancel }) => {
				if ((this, this.currentStep === 2)) return;
				cancel();

				const valid = await this.addPersonnelFormMeta.validateForm({ update: true });
				if (valid) {
					this.currentStep += 1;
					this.askPin = true;
				}
			},
			onUpdate: async ({ result, form, cancel }) => {
				if (result.type === 'success') {
					const resInvokeHospitalAdminAddActivationKey = await tryCatchAsVal(async () => {
						return (await invoke('hospital_admin_add_activation_key', {
							personnelIdPart: form.data.id,
							role: form.data.role,
							pin: form.data.pin
						})) as SuccessResponse<InvokeHospitalAdminAddActivationKeyResponse>;
					});

					if (!resInvokeHospitalAdminAddActivationKey.success) {
						cancel();
						console.log(resInvokeHospitalAdminAddActivationKey.error);
						toast.error(resInvokeHospitalAdminAddActivationKey.error);
						return;
					}

					this.askPin = false;
					this.addPersonnelDialogOpen = false;
					this.currentStep = 1;

					this.refetchPersonnels = this.getHospitalPersonnels();
				}
			}
		});

		// this is magic tho :)
		this.addPersonnelFormMeta.form.subscribe((val) => (this.something = val));
	}

	getHospitalPersonnels = async () => {
		const resInvokeGetHospitalPersonnels = await tryCatchAsVal(async () => {
			return (await invoke('get_hospital_personnels')) as SuccessResponse<{
				personnels: HospitalPersonnel[];
			}>;
		});

		if (!resInvokeGetHospitalPersonnels.success) {
			toast.error(resInvokeGetHospitalPersonnels.error);
			return [];
		}

		return resInvokeGetHospitalPersonnels.data.data.personnels;
	};

	refetchPersonnels = $state<Promise<HospitalPersonnel[]>>(this.getHospitalPersonnels());
}
