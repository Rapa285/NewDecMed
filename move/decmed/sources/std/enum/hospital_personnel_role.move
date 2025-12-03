module decmed::std_enum_hospital_personnel_role;

public enum HospitalPersonnelRole has copy, drop, store {
    Admin,
    AdministrativePersonnel,
    MedicalPersonnel,
}

public(package) fun admin(): HospitalPersonnelRole {
    HospitalPersonnelRole::Admin
}

public(package) fun administrative_personnel(): HospitalPersonnelRole {
    HospitalPersonnelRole::AdministrativePersonnel
}

public(package) fun medical_personnel(): HospitalPersonnelRole {
    HospitalPersonnelRole::MedicalPersonnel
}
