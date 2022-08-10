use std::{
    ffi::OsStr,
    iter::once,
    mem::{size_of, transmute},
    os::windows::ffi::OsStrExt,
    ptr::null_mut,
};

use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::ERROR_SUCCESS,
        System::Registry::{
            RegCloseKey, RegCreateKeyExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER, KEY_WRITE,
            REG_DWORD, REG_OPTION_NON_VOLATILE,
        },
    },
};

fn os_str(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(once(0)).collect()
}

pub struct RegistryKey {
    predefined: bool,
    hkey: HKEY,
}

impl RegistryKey {
    pub const HKCU: Self = Self {
        predefined: true,
        hkey: HKEY_CURRENT_USER,
    };

    pub fn open_or_create(parent_key: &Self, sub_key: &str) -> Self {
        let mut hkey = HKEY::default();

        let status = unsafe {
            RegCreateKeyExW(
                parent_key.hkey,
                PCWSTR(os_str(sub_key).as_ptr()),
                0,
                None,
                REG_OPTION_NON_VOLATILE,
                KEY_WRITE,
                null_mut(),
                &mut hkey,
                null_mut(),
            )
        };

        assert_eq!(status, ERROR_SUCCESS, "Error opening or creating new key");

        Self {
            predefined: false,
            hkey,
        }
    }

    pub fn set_dword(&self, value: &str, data: u32) {
        let status = unsafe {
            RegSetValueExW(
                self.hkey,
                PCWSTR(os_str(value).as_ptr()),
                0,
                REG_DWORD,
                transmute(&data as *const u32),
                size_of::<u32>() as u32,
            )
        };

        assert_eq!(status, ERROR_SUCCESS, "Error setting the key value");
    }
}

impl Drop for RegistryKey {
    fn drop(&mut self) {
        if !self.predefined {
            unsafe { RegCloseKey(self.hkey) };
        }
    }
}
