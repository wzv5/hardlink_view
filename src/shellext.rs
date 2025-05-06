use std::{cell::RefCell, path::PathBuf};

use windows::{
    Win32::{
        Foundation::{CLASS_E_NOAGGREGATION, E_NOTIMPL, E_UNEXPECTED, HINSTANCE, HWND, LPARAM},
        System::{
            Com::{
                DVASPECT_CONTENT, FORMATETC, IClassFactory, IClassFactory_Impl, IDataObject,
                TYMED_HGLOBAL,
            },
            Ole::{CF_HDROP, ReleaseStgMedium},
            Registry::HKEY,
        },
        UI::{
            Controls::{
                CreatePropertySheetPageW, DestroyPropertySheetPage, LPFNSVADDPROPSHEETPAGE,
                PROPSHEETPAGEW, PSP_USECALLBACK, PSP_USETITLE, PSPCB_MESSAGE, PSPCB_RELEASE,
            },
            Shell::{
                Common::ITEMIDLIST, DragQueryFileW, HDROP, IShellExtInit, IShellExtInit_Impl,
                IShellPropSheetExt, IShellPropSheetExt_Impl,
            },
        },
    },
    core::{BOOL, GUID, HSTRING, IUnknown, Interface, PCWSTR, Ref, Result, implement},
};

use crate::{dlg, hardlink, resource, util::VecU16StringExt};

// {552DB720-103A-4783-9EB4-834E5E5915BE}
pub const HARDLINKVIEW_CLSID: GUID = GUID::from_u128(0x552db720_103a_4783_9eb4_834e5e5915be);

#[implement(IShellExtInit, IShellPropSheetExt)]
#[derive(Default)]
struct HardlinkView {
    files: RefCell<Vec<(PathBuf, Vec<PathBuf>)>>,
}

impl IShellExtInit_Impl for HardlinkView_Impl {
    fn Initialize(
        &self,
        _pidlfolder: *const ITEMIDLIST,
        pdtobj: Ref<'_, IDataObject>,
        _hkeyprogid: HKEY,
    ) -> Result<()> {
        unsafe {
            let fe = FORMATETC {
                cfFormat: CF_HDROP.0,
                dwAspect: DVASPECT_CONTENT.0,
                lindex: -1,
                tymed: TYMED_HGLOBAL.0 as _,
                ptd: std::ptr::null_mut(),
            };
            if let Some(mut medium) = pdtobj.as_ref().and_then(|obj| obj.GetData(&fe).ok()) {
                let count = DragQueryFileW(HDROP(medium.u.hGlobal.0), u32::MAX, None);
                let mut buff = Vec::<u16>::new();
                for i in 0..count {
                    let cch = DragQueryFileW(HDROP(medium.u.hGlobal.0), i, None);
                    if cch > 0 {
                        buff.resize((cch + 1) as _, 0);
                        DragQueryFileW(HDROP(medium.u.hGlobal.0), i, Some(&mut buff));
                        let p = PathBuf::from(buff.to_os_string());
                        if p.is_file() {
                            if let Ok(links) = hardlink::get_hardlink(&p) {
                                if !links.is_empty() {
                                    self.files.borrow_mut().push((p, links));
                                }
                            }
                        }
                    }
                }
                ReleaseStgMedium(&mut medium);
            }
        }
        self.files.borrow_mut().sort_by(|a, b| a.0.cmp(&b.0));
        Ok(())
    }
}

impl IShellPropSheetExt_Impl for HardlinkView_Impl {
    fn AddPages(&self, pfnaddpage: LPFNSVADDPROPSHEETPAGE, lparam: LPARAM) -> Result<()> {
        if pfnaddpage.is_none() {
            return E_UNEXPECTED.ok();
        }
        let pfnaddpage = pfnaddpage.unwrap();
        if self.files.borrow().is_empty() {
            return Ok(());
        }
        // TODO: 支持多个文件
        if self.files.borrow().len() > 1 {
            return Ok(());
        }
        let title = HSTRING::from("硬链接");
        let mut psp = PROPSHEETPAGEW::default();
        psp.dwSize = std::mem::size_of::<PROPSHEETPAGEW>() as _;
        psp.dwFlags = PSP_USETITLE | PSP_USECALLBACK;
        psp.hInstance = HINSTANCE(*crate::dll::DLL_INSTANCE.get().unwrap() as _);
        psp.pszTitle = PCWSTR(title.as_ptr());
        psp.Anonymous1.pszTemplate = PCWSTR(resource::IDD_HARDLINKVIEW_PROPPAGE as _);
        psp.pfnDlgProc = Some(dlg::dlg_proc);
        psp.pfnCallback = Some(psp_cb);
        let (path, links) = self.files.take().pop().unwrap();
        let dlg = Box::new(dlg::Dialog::new(path, links));
        psp.lParam = LPARAM(Box::into_raw(dlg) as _);
        unsafe {
            let page = CreatePropertySheetPageW(&mut psp);
            if !pfnaddpage(page, lparam).as_bool() {
                let _ = DestroyPropertySheetPage(page);
                let _ = Box::from_raw(psp.lParam.0 as *mut dlg::Dialog);
            }
        }
        Ok(())
    }

    fn ReplacePage(
        &self,
        _upageid: u32,
        _pfnreplacewith: LPFNSVADDPROPSHEETPAGE,
        _lparam: LPARAM,
    ) -> Result<()> {
        E_NOTIMPL.ok()
    }
}

#[implement(IClassFactory)]
pub struct Factory;

impl IClassFactory_Impl for Factory_Impl {
    fn CreateInstance(
        &self,
        punkouter: Ref<'_, IUnknown>,
        riid: *const GUID,
        ppvobject: *mut *mut core::ffi::c_void,
    ) -> Result<()> {
        if punkouter.is_some() {
            return CLASS_E_NOAGGREGATION.ok();
        }
        let unknown: IUnknown = HardlinkView::default().into();
        unsafe { unknown.query(riid, ppvobject).ok() }
    }

    fn LockServer(&self, _flock: BOOL) -> Result<()> {
        E_NOTIMPL.ok()
    }
}

unsafe extern "system" fn psp_cb(
    _hwnd: HWND,
    umsg: PSPCB_MESSAGE,
    ppsp: *mut PROPSHEETPAGEW,
) -> u32 {
    if umsg == PSPCB_RELEASE && !ppsp.is_null() {
        let psp = unsafe { &*ppsp };
        if psp.lParam.0 != 0 {
            let _ = unsafe { Box::from_raw(psp.lParam.0 as *mut dlg::Dialog) };
        }
    }
    1
}
