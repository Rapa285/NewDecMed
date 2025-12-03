import type { completeProfileSchema, signInSchemaStep3, signUpSchemaStep4 } from './schema';

export type Account = {
	id: string;
	name: string;
};

export type CompleteProfileSchema = typeof completeProfileSchema;

export type InvokeGetMedicalRecordsResponse = {
	cid: string;
	createdAt: string;
	index: string;
};

export type InvokeGetAccessLog = {
	access_data_type: ('Administrative' | 'Medical')[];
	access_type: 'Read' | 'Update';
	date: string;
	exp_dur: number;
	hospital_metadata: {
		name: string;
	};
	hospital_personnel_address: string;
	hospital_personnel_metadata: {
		name: string;
	};
	index: number;
	is_revoked: boolean;
};

export type InvokeProcessQrResponse = {
	hospitalPersonnelHospitalName: string;
	hospitalPersonnelName: string;
};

export type InvokeGetMedicalRecordResponse = {
	createdAt: string;
	medicalData: TauriMedicalData;
};

export type TauriAdministrativeData = {
	id: string;
	idHash: string;
	iotaAddress: string;
	prePublicKey: string;
	name: string | null;
	birthPlace: string | null;
	dateOfBirth: string | null;
	gender: string | null;
	religion: string | null;
	education: string | null;
	occupation: string | null;
	maritalStatus: string | null;
};

export type TauriMedicalData = {
	anamnesis: string;
	physical_check: string;
	psychological_check: string;
	diagnose: string;
	therapy: string;
};

export type TauriMedicalDataMainCategory = 'Category1' | 'Category2';

export type TauriMedicalDataSubCategory = 'SubCategory1' | 'SubCategory2';

export type NavLink = {
	label: string;
	link: string;
	pageTitle: string;
};

export type SignUpSchemaStep4 = typeof signUpSchemaStep4;
export type SIgnInSchemaStep3 = typeof signInSchemaStep3;

export type SuccessResponse<T> = {
	data: T;
	status: string;
};

export type TryCatchAsValError = {
	error: string;
	success: false;
};
export type TryCatchAsValReturn<T> = TryCatchAsValSuccess<T> | TryCatchAsValError;
export type TryCatchAsValSuccess<T> = {
	success: true;
	data: T;
};
