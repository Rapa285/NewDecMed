import { z } from 'zod';

export const hospitalIdField = {
	hospitalId: z
		.string()
		.trim()
		.regex(/^[a-z0-9_]{3,50}$/g, {
			message: 'Invalid hospital ID, only (a-z0-9_) 3-50 chars accepted'
		})
		.transform((val) => val.trim())
};

export const hospitalNameField = {
	hospitalName: z
		.string()
		.trim()
		.regex(/^[a-zA-Z0-9 ]{3,50}$/g, {
			message: 'Invalid hospital name, only (a-zA-Z0-9 ) 3-50 chars accepted'
		})
		.transform((val) => val.trim())
};

export const addHospitalSchema = z.object(hospitalIdField).extend(hospitalNameField);
