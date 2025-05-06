use std::path::{Path, PathBuf};

use windows::{
    Win32::{
        Foundation::{ERROR_HANDLE_EOF, ERROR_MORE_DATA, MAX_PATH},
        Storage::FileSystem::{FindClose, FindFirstFileNameW, FindNextFileNameW},
    },
    core::{HSTRING, PCWSTR, PWSTR, Result},
};

use crate::util::VecU16StringExt;

pub fn get_hardlink(filename: &Path) -> Result<Vec<PathBuf>> {
    let prefix = filename
        .components()
        .next()
        .and_then(|c| {
            if matches!(c, std::path::Component::Prefix(_)) {
                Some(AsRef::<Path>::as_ref(&c).to_owned())
            } else {
                None
            }
        })
        .ok_or(windows::Win32::Foundation::ERROR_PATH_NOT_FOUND)?;
    let mut result = vec![];
    let mut buff = Vec::<u16>::new();
    buff.resize((MAX_PATH + 1) as _, 0);
    let mut cch: u32 = buff.len() as _;
    let filenamew = HSTRING::from(filename);
    let r = unsafe {
        FindFirstFileNameW(
            PCWSTR(filenamew.as_ptr()),
            0,
            &mut cch,
            PWSTR(buff.as_mut_ptr()),
        )
    };
    let h = match r {
        Err(e) if e.code() == ERROR_MORE_DATA.to_hresult() => {
            buff.resize((cch + 1) as _, 0);
            unsafe {
                FindFirstFileNameW(
                    PCWSTR(filenamew.as_ptr()),
                    0,
                    &mut cch,
                    PWSTR(buff.as_mut_ptr()),
                )
            }?
        }
        Err(e) => return Err(e),
        Ok(h) => h,
    };
    result.push(prefix.join(buff.to_os_string()));
    loop {
        cch = buff.len() as _;
        match unsafe { FindNextFileNameW(h, &mut cch, PWSTR(buff.as_mut_ptr())) } {
            Err(e) if e.code() == ERROR_MORE_DATA.to_hresult() => {
                buff.resize((cch + 1) as _, 0);
            }
            Err(e) if e.code() == ERROR_HANDLE_EOF.to_hresult() => {
                break;
            }
            Err(e) => return Err(e),
            Ok(_) => result.push(prefix.join(buff.to_os_string())),
        };
    }
    unsafe { FindClose(h)? };
    if let Some(i) = result.iter().position(|p| p == filename) {
        result.swap_remove(i);
    }
    result.sort();
    Ok(result)
}
