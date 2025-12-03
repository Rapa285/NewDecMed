import { goto } from '$app/navigation';
import { createMedicalRecordSchema } from '$lib/schema';
import type {
	CreateMedicalRecordSchema,
	InvokeGetPatientAdministrativeDataResponseData,
	SuccessResponse
} from '$lib/types';
import { tryCatchAsVal } from '$lib/utils';
import { invoke } from '@tauri-apps/api/core';
import { toast } from 'svelte-sonner';
import { superForm, type Infer, type SuperForm, type SuperValidated } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';

type Props = {
	accessToken: string;
	patientIotaAddress: string;
	patientPrePublicKey: string;
	createMedicalRecordForm: SuperValidated<Infer<CreateMedicalRecordSchema>>;
};

export class EmrCreateState {
	accessToken = $state<string>('');
	patientIotaAddress = $state('');
	patientPrePublicKey = $state<string>('');
	createMedicalRecordFormMeta: SuperForm<Infer<CreateMedicalRecordSchema>>;

	constructor({
		accessToken,
		createMedicalRecordForm,
		patientIotaAddress,
		patientPrePublicKey
	}: Props) {
		this.accessToken = accessToken;
		this.patientIotaAddress = patientIotaAddress;
		this.patientPrePublicKey = patientPrePublicKey;

		this.createMedicalRecordFormMeta = superForm(createMedicalRecordForm, {
			validators: zod(createMedicalRecordSchema),
			dataType: 'json',
			SPA: true,
			invalidateAll: false,
			onUpdate: async ({ form, result, cancel }) => {
				if (result.type === 'success') {
					const resInvokeCreateMedicalRecord = await tryCatchAsVal(async () => {
						return (await invoke('new_medical_record', {
							accessToken,
							data: {
								anamnesis: form.data.anamnesis,
								physicalCheck: form.data.physicalCheck,
								psychologicalCheck: form.data.psychologicalCheck,
								diagnose: form.data.diagnose,
								therapy: form.data.therapy
							},
							patientIotaAddress,
							patientPrePublicKey
						})) as SuccessResponse<null>;
					});

					if (!resInvokeCreateMedicalRecord.success) {
						toast.error(resInvokeCreateMedicalRecord.error);
						cancel();
						return;
					}

					toast.success('Medical record created sucessfully');
					await goto('/dashboard');
				}
			}
		});
	}

	getPatientAdministrativeData = async (accessToken: string | null, patientIotaAddress: string) => {
		const resInvokeGetPatientAdministrativeData = await tryCatchAsVal(async () => {
			return (await invoke('get_administrative_data', {
				accessToken,
				patientIotaAddress
			})) as SuccessResponse<InvokeGetPatientAdministrativeDataResponseData>;
		});

		console.log(resInvokeGetPatientAdministrativeData);

		if (!resInvokeGetPatientAdministrativeData.success) {
			toast.error(resInvokeGetPatientAdministrativeData.error);
			throw new Error(resInvokeGetPatientAdministrativeData.error);
		}

		return resInvokeGetPatientAdministrativeData.data.data;
	};

	fetchPatientAdministrativeData = $derived(
		this.getPatientAdministrativeData(this.accessToken, this.patientIotaAddress)
	);
}
