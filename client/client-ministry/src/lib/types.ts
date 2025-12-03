import { addHospitalSchema } from './schema';

export type AddHospitalSchema = typeof addHospitalSchema;

export type InvokeGetHospitalsResponseData = {
	activationKey: string;
	hospitalAdminCid: string;
	hospitalName: string;
};

export type SuccessResponse<T> = {
	status: string;
	data: T;
};

export type TryCatchAsValSuccess<T> = { success: true; data: T };
export type TryCatchAsValError = { success: false; error: string };
export type TryCatchAsValReturn<T> = TryCatchAsValSuccess<T> | TryCatchAsValError;
