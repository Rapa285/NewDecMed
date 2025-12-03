import { z } from 'zod';
import { ADMINISTRATIVE_PERSONNEL_ROLE, MEDICAL_PERSONNEL_ROLE } from './constants';

const pinSchema = {
	pin: z
		.string({
			required_error: 'PIN is required.',
			invalid_type_error: 'PIN is invalid.'
		})
		.trim()
		.regex(/^\d{6}$/, { message: 'PIN is invalid.' })
		.min(1, { message: 'PIN is required.' })
		.max(6, { message: 'PIN maximum 6 digits.' })
		.transform((val) => val.trim())
};

const nameSchema = {
	name: z
		.string({ required_error: 'Name is required.', invalid_type_error: 'Name is invalid.' })
		.trim()
		.regex(/^[a-zA-Z0-9 ]{2,100}$/, {
			message: 'Name must consist of alphanumeric characters only of length 2 - 100.'
		})
		.transform((val) => val.trim())
};

export const medicalDataMainCategory = {
	mainCategory: z.enum(['Category1', 'Category2'])
};

export const medicalDataSubCategory = {
	subCategory: z.enum(['SubCategory1', 'SubCategory2'])
};

const anamnesisSchema = {
	anamnesis: z
		.string({
			required_error: 'Anamnesis is required.',
			invalid_type_error: 'Anamnesis is invalid.'
		})
		.trim()
		.regex(/^[a-zA-Z0-9:,.\\ ]{2,1000}$/, {
			message: 'Anamnesis must consist of alphanumeric characters only of length 2 - 100.'
		})
		.transform((val) => val.trim())
};

const physicalCheckSchema = {
	physicalCheck: z
		.string({
			required_error: 'Physical check is required.',
			invalid_type_error: 'Physical check is invalid.'
		})
		.trim()
		.regex(/^[a-zA-Z0-9:,.\\ ]{2,1000}$/, {
			message: 'Physical check must consist of alphanumeric characters only of length 2 - 100.'
		})
		.transform((val) => val.trim())
};

const psychologicalCheckSchema = {
	psychologicalCheck: z
		.string({
			required_error: 'Psychological check is required.',
			invalid_type_error: 'Psychological check is invalid.'
		})
		.trim()
		.regex(/^[a-zA-Z0-9:,.\\ ]{2,1000}$/, {
			message: 'Psychological check must consist of alphanumeric characters only of length 2 - 100.'
		})
		.transform((val) => val.trim())
};

const diagnoseSchema = {
	diagnose: z
		.string({
			required_error: 'Diagnose is required.',
			invalid_type_error: 'Diagnose is invalid.'
		})
		.trim()
		.regex(/^[a-zA-Z0-9:,.\\ ]{2,1000}$/, {
			message: 'Diagnose must consist of alphanumeric characters only of length 2 - 100.'
		})
		.transform((val) => val.trim())
};

const therapySchema = {
	therapy: z
		.string({
			required_error: 'Therapy is required.',
			invalid_type_error: 'Therapy is invalid.'
		})
		.trim()
		.regex(/^[a-zA-Z0-9:,.\\ ]{2,1000}$/, {
			message: 'Therapy must consist of alphanumeric characters only of length 2 - 100.'
		})
		.transform((val) => val.trim())
};

// const _hospitalSchema = {
// 	hospital: z
// 		.string({ required_error: 'Hospital is required.', invalid_type_error: 'Hospital is invalid.' })
// 		.trim()
// 		.regex(/^[a-zA-Z0-9 ]{2,100}$/, {
// 			message: 'Hospital must consist of alphanumeric characters only of length 2 - 100.'
// 		})
// 		.transform((val) => val.trim())
// };

export const activationSchema = z.object({
	id: z
		.string({
			required_error: 'ID is required.',
			invalid_type_error: 'ID is invalid.'
		})
		.trim()
		.min(1, { message: 'ID is required.' })
		.transform((val) => val.trim()),
	activationKey: z
		.string({
			required_error: 'Activation Key is required.',
			invalid_type_error: 'Activation Key is invalid.'
		})
		.trim()
		.min(1, { message: 'Activation Key is required.' })
		.max(36, { message: 'Activation Key is invalid.' })
		.transform((val) => val.trim())
});

export const signInSchemaStep1 = z.object(pinSchema);

export const signInSchemaStep2 = signInSchemaStep1.extend({
	confirmPin: z
		.string({
			required_error: 'Confirm PIN is required.',
			invalid_type_error: 'Confirm PIN is invalid.'
		})
		.trim()
		.regex(/^\d{6}$/, { message: 'Confirm PIN is invalid.' })
		.min(1, { message: 'Confirm PIN is required.' })
		.max(6, { message: 'Confirm PIN maximum 6 digits.' })
		.transform((val) => val.trim())
});

export const signInSchemaStep3 = signInSchemaStep2
	.extend({
		seedWords: z
			.string({
				required_error: 'Seed Words is required.',
				invalid_type_error: 'Seed Words is invalid.'
			})
			.trim()
			.min(1, { message: 'Seed Words is required.' })
			.transform((val) => val.trim())
			.refine(
				(val) => {
					const words = val.split(' ');
					return words.length === 12;
				},
				{
					message: 'Seed Words is invalid.'
				}
			)
	})
	.superRefine((val, ctx) => {
		if (val.pin !== val.confirmPin) {
			ctx.addIssue({
				code: z.ZodIssueCode.custom,
				path: ['confirmPin'],
				message: 'PIN and Confirm PIN must be same.'
			});
		}
	});

export const signUpSchemaStep1 = signInSchemaStep1;
export const signUpSchemaStep2 = signInSchemaStep2;
export const signUpSchemaStep4 = signInSchemaStep3;

export const addPersonnelSchemaStep1 = z.object({
	id: z
		.string({
			required_error: 'ID is required.',
			invalid_type_error: 'ID is invalid.'
		})
		.trim()
		.min(1, { message: 'ID is required.' })
		.transform((val) => val.trim()),
	role: z.enum([ADMINISTRATIVE_PERSONNEL_ROLE, MEDICAL_PERSONNEL_ROLE], {
		required_error: 'Role is required.',
		invalid_type_error: 'Role is invalid.'
	})
});

export const addPersonnelSchemaStep2 = addPersonnelSchemaStep1.extend(pinSchema);
export const completeProfileAdminSchema = z.object(nameSchema);
export const completeProfilePersonnelSchema = z.object(nameSchema);
export const createMedicalRecordSchema = z
	.object(anamnesisSchema)
	.extend(physicalCheckSchema)
	.extend(psychologicalCheckSchema)
	.extend(diagnoseSchema)
	.extend(therapySchema);
export const updateMedicalRecordSchema = z
	.object(anamnesisSchema)
	.extend(physicalCheckSchema)
	.extend(psychologicalCheckSchema)
	.extend(diagnoseSchema)
	.extend(therapySchema);

export const addPersonnelSchemas = [addPersonnelSchemaStep1, addPersonnelSchemaStep2];
export const signInSchemas = [signInSchemaStep1, signInSchemaStep2, signInSchemaStep3];
export const signUpSchemas = [
	signUpSchemaStep1,
	signUpSchemaStep2,
	signUpSchemaStep2,
	signUpSchemaStep4
];
