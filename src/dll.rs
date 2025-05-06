#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use std::{path::PathBuf, sync::OnceLock};

use windows::{
    Win32::{
        Foundation::{
            CLASS_E_CLASSNOTAVAILABLE, E_UNEXPECTED, ERROR_INSUFFICIENT_BUFFER, GetLastError,
            HINSTANCE, HMODULE, MAX_PATH, S_OK,
        },
        System::{
            Com::{CLSCTX_INPROC_SERVER, CoCreateInstance, IClassFactory},
            LibraryLoader::GetModuleFileNameW,
            SystemServices::DLL_PROCESS_ATTACH,
        },
    },
    core::{GUID, HRESULT, HSTRING, IUnknown, IUnknown_Vtbl, Interface, PCWSTR, interface, w},
};

use crate::{
    logger,
    resource::IDR_HARDLINKVIEW_RGS,
    shellext::{Factory, HARDLINKVIEW_CLSID},
    util::VecU16StringExt,
};

pub static DLL_INSTANCE: OnceLock<usize> = OnceLock::new();
pub static DLL_PATH: OnceLock<PathBuf> = OnceLock::new();

const CLSID_Registrar: GUID = GUID::from_u128(0x44ec053a_400f_11d0_9dcd_00a0c90391d3);

#[interface("44ec053b-400f-11d0-9dcd-00a0c90391d3")]
unsafe trait IRegistrar: IUnknown {
    fn AddReplacement(&self, key: PCWSTR, item: PCWSTR) -> HRESULT;
    fn _placeholder1(&self);
    fn _placeholder2(&self);
    fn _placeholder3(&self);
    fn _placeholder4(&self);
    fn _placeholder5(&self);
    fn _placeholder6(&self);
    fn _placeholder7(&self);
    fn ResourceRegister(&self, resFileName: PCWSTR, nID: u32, szType: PCWSTR) -> HRESULT;
    fn ResourceUnregister(&self, resFileName: PCWSTR, nID: u32, szType: PCWSTR) -> HRESULT;
}

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

fn register_class_object(register: bool) -> HRESULT {
    unsafe {
        let registrar: IRegistrar =
            match CoCreateInstance(&CLSID_Registrar, None, CLSCTX_INPROC_SERVER) {
                Ok(obj) => obj,
                Err(err) => return err.code(),
            };
        let dll_path = HSTRING::from(DLL_PATH.get().unwrap().as_os_str());
        let hr = registrar.AddReplacement(w!("MODULE"), PCWSTR(dll_path.as_ptr()));
        if hr != S_OK {
            return hr;
        }
        let hr = if register {
            registrar.ResourceRegister(
                PCWSTR(dll_path.as_ptr()),
                IDR_HARDLINKVIEW_RGS,
                w!("REGISTRY"),
            )
        } else {
            registrar.ResourceUnregister(
                PCWSTR(dll_path.as_ptr()),
                IDR_HARDLINKVIEW_RGS,
                w!("REGISTRY"),
            )
        };
        return hr;
    }
}

#[unsafe(no_mangle)]
unsafe extern "system" fn DllRegisterServer() -> HRESULT {
    register_class_object(true)
}

#[unsafe(no_mangle)]
unsafe extern "system" fn DllUnregisterServer() -> HRESULT {
    register_class_object(false)
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
