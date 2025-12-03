module decmed::std_enum_hospital_personnel_access_type;

public enum HospitalPersonnelAccessType has copy, drop, store {
    Read,
    Update,
}

public(package) fun read(): HospitalPersonnelAccessType {
    HospitalPersonnelAccessType::Read
}

public(package) fun update(): HospitalPersonnelAccessType {
    HospitalPersonnelAccessType::Update
}
