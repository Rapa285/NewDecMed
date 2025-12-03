import { SIGNUP_TOTAL_STEP } from '$lib/constants';
import { signUpSchemas } from '$lib/schema';
import type { SignUpSchemaStep4, SuccessResponse } from '$lib/types';
import { invoke } from '@tauri-apps/api/core';
import { superForm, type Infer, type SuperForm, type SuperValidated } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';
import { getAuthContext } from '../../(context)/auth-context.svelte';
import { tryCatchAsVal } from '$lib/utils';
import { toast } from 'svelte-sonner';
import { invalidate, invalidateAll } from '$app/navigation';

type Constructor = {
	signUpForm: SuperValidated<Infer<SignUpSchemaStep4>>;
};

export class SignUpState {
	currentStep = $state<number>(1);
	mnemonic = $state<string>('');
	signUpFormMeta: SuperForm<Infer<SignUpSchemaStep4>>;
	authContext = getAuthContext();

	constructor({ signUpForm }: Constructor) {
		$effect(() => {
			this.signUpFormMeta.options.validators = zod(signUpSchemas[this.currentStep - 1]);
		});

		this.signUpFormMeta = superForm(signUpForm, {
			validators: false,
			dataType: 'json',
			SPA: true,
			onSubmit: async ({ cancel, formData }) => {
				if (this.currentStep === SIGNUP_TOTAL_STEP) return;
				cancel();

				const result = await this.signUpFormMeta.validateForm({ update: true });
				let valid = true;

				if (result.valid) {
					switch (this.currentStep) {
						case 1: {
							const pin = formData.get('pin') as string;
							const resInvokeValidatePin = await tryCatchAsVal(async () => {
								return (await invoke('validate_pin', {
									pin,
									authType: 'Signup'
								})) as SuccessResponse<null>;
							});

							valid = resInvokeValidatePin.success;

							if (!resInvokeValidatePin.success) {
								this.signUpFormMeta.errors.update((val) => {
									val.pin = [resInvokeValidatePin.error];
									return val;
								});
							}

							break;
						}
						case 2: {
							const confirmPin = formData.get('confirmPin') as string;
							const resInvokeValidateConfirmPin = await tryCatchAsVal(async () => {
								return (await invoke('validate_confirm_pin', {
									confirmPin,
									authType: 'Signup'
								})) as SuccessResponse<null>;
							});

							valid = resInvokeValidateConfirmPin.success;

							if (!resInvokeValidateConfirmPin.success) {
								this.signUpFormMeta.errors.update((val) => {
									val.confirmPin = [resInvokeValidateConfirmPin.error];
									return val;
								});
							}

							break;
						}
					}
				} else {
					valid = false;
				}

				if (valid) this.currentStep += 1;

				if (this.currentStep == 3) {
					const resInvokeGenerateMnemonic = await tryCatchAsVal(async () => {
						return (await invoke('generate_mnemonic')) as SuccessResponse<string>;
					});

					if (!resInvokeGenerateMnemonic.success) {
						toast.error(resInvokeGenerateMnemonic.error);
						invalidateAll();
					} else {
						this.mnemonic = resInvokeGenerateMnemonic.data.data;
					}
				}
			},
			onUpdate: async ({ form, result, cancel }) => {
				if (result.type === 'success') {
					const resInvokeSignup = await tryCatchAsVal(async () => {
						return (await invoke('signup', {
							seedWords: form.data.seedWords
						})) as SuccessResponse<null>;
					});

					if (!resInvokeSignup.success) {
						cancel();
						toast.error(resInvokeSignup.error);
					}
				}
			}
		});
	}

	copyMnemonic = async () => {
		await navigator.clipboard.writeText(this.mnemonic);
	};
}
