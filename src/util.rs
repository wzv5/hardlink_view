use std::ffi::OsString;

use windows::{
    Win32::{Foundation::HINSTANCE, UI::WindowsAndMessaging::LoadStringW},
    core::PWSTR,
};

use crate::dll::DLL_INSTANCE;

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

pub fn load_string(uid: u32) -> OsString {
    unsafe {
        let hdll = HINSTANCE(*DLL_INSTANCE.get().unwrap() as _);
        let mut p = 0usize;
        let n = LoadStringW(Some(hdll), uid, PWSTR(&mut p as *mut usize as _), 0);
        if n > 0 {
            return widestring::U16CStr::from_ptr_str(p as _).to_os_string();
        }
    }
    OsString::new()
}
