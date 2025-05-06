use std::ffi::OsString;

pub trait VecU16StringExt {
    fn into_os_string(self) -> OsString;
    fn to_os_string(&self) -> OsString;
}

impl VecU16StringExt for Vec<u16> {
    fn into_os_string(self) -> OsString {
        widestring::U16CString::from_vec_truncate(self).to_os_string()
    }

    fn to_os_string(&self) -> OsString {
        self.clone().into_os_string()
    }
}
