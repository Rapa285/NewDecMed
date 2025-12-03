import { z } from 'zod';
import { JPEG_MIME_TYPE, ONE_MB, PNG_MIME_TYPE } from './constants';

const confirmPinSchema = {
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

const birthPlaceSchema = {
	birthPlace: z
		.string({
			required_error: 'Birth place is required.',
			invalid_type_error: 'Birth place is invalid.'
		})
		.trim()
		.regex(/^[a-zA-Z0-9 ]{2,50}$/, {
			message: 'Birth place must consist of alphanumeric characters only of length 2 - 50.'
		})
		.transform((val) => val.trim())
};

const dateOfBirthSchema = {
	dateOfBirth: z
		.string({
			required_error: 'Date of birth place is required.',
			invalid_type_error: 'Date of birth place is invalid.'
		})
		.trim()
		.regex(/^\d{4}-\d{2}-\d{2}$/, {
			message: 'Date of birth is invalid.'
		})
		.transform((val) => val.trim())
};

const genderSchema = {
	gender: z.enum(['Male', 'Female'], { message: 'Invalid gender.' })
};

const religionSchema = {
	religion: z
		.string({
			required_error: 'Religion is required.',
			invalid_type_error: 'Religion is invalid.'
		})
		.trim()
		.regex(/^[a-zA-Z0-9 ]{2,50}$/, {
			message: 'Religion must consist of alphanumeric characters only of length 2 - 50.'
		})
		.transform((val) => val.trim())
};

const educationSchema = {
	education: z
		.string({
			required_error: 'Education is required.',
			invalid_type_error: 'Education is invalid.'
		})
		.trim()
		.regex(/^[a-zA-Z0-9 ]{2,50}$/, {
			message: 'Education must consist of alphanumeric characters only of length 2 - 50.'
		})
		.transform((val) => val.trim())
};

const occupationSchema = {
	occupation: z
		.string({
			required_error: 'Occupation is required.',
			invalid_type_error: 'Occupation is invalid.'
		})
		.trim()
		.regex(/^[a-zA-Z0-9 ]{2,50}$/, {
			message: 'Occupation must consist of alphanumeric characters only of length 2 - 50.'
		})
		.transform((val) => val.trim())
};

const maritalStatusSchema = {
	maritalStatus: z.enum(['Single', 'Married', 'Widowed', 'Divorced'], {
		message: 'Invalid marital status.'
	})
};

const nikSchema = {
	nik: z
		.string({
			required_error: 'NIK is required.',
			invalid_type_error: 'NIK is invalid.'
		})
		.trim()
		.regex(/^\d{16}$/, { message: 'NIK is invalid.' })
		.min(1, { message: 'NIK is required.' })
		.transform((val) => val.trim())
};

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

const seedWordsSchema = {
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
};

const qrSchema = {
	qr: z
		.instanceof(File, { message: 'Please upload a file.' })
		.refine((f) => f.size <= ONE_MB, 'Max 1 MB upload size.')
		.refine(
			(f) => [JPEG_MIME_TYPE, PNG_MIME_TYPE].includes(f.type),
			'Should be image/png or image/jpeg'
		)
};

export const completeProfileSchema = z
	.object(nameSchema)
	.extend(birthPlaceSchema)
	.extend(dateOfBirthSchema)
	.extend(genderSchema)
	.extend(religionSchema)
	.extend(educationSchema)
	.extend(occupationSchema)
	.extend(maritalStatusSchema);

export const enterPinSchema = z.object(pinSchema);

export const hospitalQrSchema = z.object(qrSchema);

export const signInSchemaStep1 = z.object(pinSchema);
export const signInSchemaStep2 = signInSchemaStep1.extend(confirmPinSchema);
export const signInSchemaStep3 = signInSchemaStep2
	.extend(seedWordsSchema)
	.extend(nikSchema)
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
export const signUpSchemaStep4 = z.object(seedWordsSchema);
export const signUpSchemaStep5 = signInSchemaStep3;

export const signInSchemas = [signInSchemaStep1, signInSchemaStep2, signInSchemaStep3];
export const signUpSchemas = [
	signUpSchemaStep1,
	signUpSchemaStep2,
	signUpSchemaStep2,
	signUpSchemaStep4,
	signUpSchemaStep5
];
