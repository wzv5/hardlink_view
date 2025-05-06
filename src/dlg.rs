use std::path::PathBuf;

use windows::Win32::{
    Foundation::{HWND, LPARAM, RECT, WPARAM},
    UI::{
        Controls::PROPSHEETPAGEW,
        WindowsAndMessaging::{
            GWLP_USERDATA, GetClientRect, GetDlgItem, GetWindowLongPtrW, MoveWindow,
            SetWindowLongPtrW, SetWindowTextW, WM_INITDIALOG, WM_SIZE,
        },
    },
};
use windows_core::HSTRING;

use crate::resource;

pub unsafe extern "system" fn dlg_proc(
    hwnd: HWND,
    msg: u32,
    _wparam: WPARAM,
    lparam: LPARAM,
) -> isize {
    match msg {
        WM_INITDIALOG => unsafe {
            let psp = &*(lparam.0 as *const PROPSHEETPAGEW);
            let this = &*(psp.lParam.0 as *const Dialog);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, this as *const _ as _);
            if let Ok(edit1) = GetDlgItem(Some(hwnd), resource::IDC_EDIT1) {
                let _ = SetWindowTextW(edit1, &HSTRING::from(this.to_string()));
            }
            return 0;
        },
        WM_SIZE => unsafe {
            if let Ok(edit1) = GetDlgItem(Some(hwnd), resource::IDC_EDIT1) {
                let mut rc = RECT {
                    left: 0,
                    top: 0,
                    right: 32,
                    bottom: 32,
                };
                let _ = GetClientRect(hwnd, &mut rc);
                let _ = MoveWindow(edit1, 16, 16, rc.right - 32, rc.bottom - 32, true);
            }
            return 0;
        },
        _ => unsafe {
            let p = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
            if p != 0 {
                let _dlg = &*(p as *mut Dialog);
            }
            return 0;
        },
    }
}

pub struct Dialog {
    path: PathBuf,
    links: Vec<PathBuf>,
}

impl Dialog {
    pub fn new(path: PathBuf, links: Vec<PathBuf>) -> Self {
        log::warn!("Dialog 创建 {}", path.to_string_lossy());
        Self { path, links }
    }

    fn to_string(&self) -> String {
        let mut s = format!(
            "{}\r\n除自身外，还有 {} 个硬链接：\r\n\r\n",
            self.path.to_string_lossy(),
            self.links.len()
        );
        for (i, l) in self.links.iter().enumerate() {
            s.push_str(format!("[{}] {}\r\n", i + 1, l.to_string_lossy()).as_str());
        }
        s
    }
}

impl Drop for Dialog {
    fn drop(&mut self) {
        log::warn!("Dialog 析构 {}", self.path.to_string_lossy());
    }
}
