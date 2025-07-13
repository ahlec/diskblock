pub struct DbUuid {}

fn uuid_to_string(uuid: &CFUUID) -> String {
    unsafe {
        let cfstr_ref = CFUUIDCreateString(ptr::null(), uuid.as_concrete_TypeRef());
        let cfstr = CFString::wrap_under_create_rule(cfstr_ref);
        cfstr.to_string()
    }
}

impl std::fmt::Display for DbUuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", uuid_to_string(self))
    }
}
