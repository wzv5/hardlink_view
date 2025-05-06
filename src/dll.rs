use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

use windows::{
    Win32::{
        Foundation::{
            CLASS_E_CLASSNOTAVAILABLE, E_UNEXPECTED, ERROR_INSUFFICIENT_BUFFER, GetLastError,
            HINSTANCE, HMODULE, MAX_PATH, S_OK,
        },
        System::{
            Com::IClassFactory, LibraryLoader::GetModuleFileNameW,
            SystemServices::DLL_PROCESS_ATTACH,
        },
        UI::Shell::{SHCNE_ASSOCCHANGED, SHCNF_IDLIST, SHChangeNotify},
    },
    core::{GUID, HRESULT, IUnknown, Interface, Result},
};

use crate::{
    logger,
    shellext::{Factory, HARDLINKVIEW_CLSID},
    util::VecU16StringExt,
};

pub static DLL_INSTANCE: OnceLock<usize> = OnceLock::new();
pub static DLL_PATH: OnceLock<PathBuf> = OnceLock::new();

#[unsafe(no_mangle)]
unsafe extern "system" fn DllGetClassObject(
    rclsid: *const GUID,
    riid: *const GUID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    unsafe {
        if *riid != IClassFactory::IID {
            return E_UNEXPECTED;
        }
        if *rclsid != HARDLINKVIEW_CLSID {
            return CLASS_E_CLASSNOTAVAILABLE;
        }
        let unknown: IUnknown = Factory.into();
        unknown.query(riid, ppv)
    }
}

fn register(clsid: &GUID, dllpath: &Path) -> Result<()> {
    let clsid = format!("{{{clsid:?}}}");
    if clsid.is_empty() {
        return E_UNEXPECTED.ok();
    }
    let hklm = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);
    let (k, _) =
        hklm.create_subkey(format!("SOFTWARE\\Classes\\CLSID\\{clsid}\\InProcServer32"))?;
    k.set_value("", &dllpath.as_os_str())?;
    k.set_value("ThreadingModel", &"Apartment")?;
    let (k, _) = hklm.create_subkey(format!(
        "SOFTWARE\\Classes\\*\\shellex\\PropertySheetHandlers\\{clsid}"
    ))?;
    k.set_value("", &clsid)?;
    Ok(())
}

fn unregister(clsid: &GUID) -> Result<()> {
    let clsid = format!("{{{clsid:?}}}");
    if clsid.is_empty() {
        return E_UNEXPECTED.ok();
    }
    let hklm = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);
    hklm.delete_subkey_all(format!("SOFTWARE\\Classes\\CLSID\\{clsid}"))?;
    hklm.delete_subkey_all(format!(
        "SOFTWARE\\Classes\\*\\shellex\\PropertySheetHandlers\\{clsid}"
    ))?;
    Ok(())
}

#[unsafe(no_mangle)]
unsafe extern "system" fn DllRegisterServer() -> HRESULT {
    if let Err(e) = register(&HARDLINKVIEW_CLSID, DLL_PATH.get().unwrap()) {
        let _ = unregister(&HARDLINKVIEW_CLSID);
        return e.into();
    }
    unsafe { SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, None, None) };
    S_OK
}

#[unsafe(no_mangle)]
unsafe extern "system" fn DllUnregisterServer() -> HRESULT {
    if let Err(e) = unregister(&HARDLINKVIEW_CLSID) {
        return e.into();
    }
    unsafe { SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, None, None) };
    S_OK
}

#[unsafe(no_mangle)]
unsafe extern "system" fn DllMain(instance: HINSTANCE, reason: u32, _: usize) -> bool {
    if reason == DLL_PROCESS_ATTACH {
        logger::init_dbg_logger();
        let mut buff = Vec::<u16>::new();
        buff.resize((MAX_PATH + 1) as _, 0);
        let cch = unsafe { GetModuleFileNameW(Some(HMODULE(instance.0)), &mut buff) };
        if cch == 0 {
            return false;
        }
        if unsafe { GetLastError() } == ERROR_INSUFFICIENT_BUFFER {
            buff.resize((cch + 1) as _, 0);
            if unsafe { GetModuleFileNameW(Some(HMODULE(instance.0)), &mut buff) } == 0 {
                return false;
            }
        }
        let _ = DLL_PATH.set(PathBuf::from(buff.into_os_string()));
        let _ = DLL_INSTANCE.set(instance.0 as _);
    }
    true
}
