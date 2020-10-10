use {
    std::{
        ffi::OsStr,
        iter::once,
        mem::{size_of, transmute},
        os::windows::ffi::OsStrExt,
        ptr::null_mut,
    },
    winapi::{
        shared::{minwindef::HKEY, winerror::ERROR_SUCCESS},
        um::{
            winnt::{KEY_WRITE, REG_DWORD, REG_OPTION_NON_VOLATILE},
            winreg::{RegCloseKey, RegCreateKeyExW, RegSetValueExW, HKEY_CURRENT_USER},
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
        let mut hkey: HKEY = null_mut();

        let status = unsafe {
            RegCreateKeyExW(
                parent_key.hkey,
                os_str(sub_key).as_ptr(),
                0,
                null_mut(),
                REG_OPTION_NON_VOLATILE,
                KEY_WRITE,
                null_mut(),
                &mut hkey,
                null_mut(),
            )
        };

        if status as u32 != ERROR_SUCCESS {
            panic!("Error opening or creating new key");
        }

        Self {
            predefined: false,
            hkey,
        }
    }

    pub fn set_dword(&self, value: &str, data: u32) {
        let status = unsafe {
            RegSetValueExW(
                self.hkey,
                os_str(value).as_ptr(),
                0,
                REG_DWORD,
                transmute(&data as *const u32),
                size_of::<u32>() as u32,
            )
        };

        if status as u32 != ERROR_SUCCESS {
            panic!("Error setting the key value");
        }
    }
}

impl Drop for RegistryKey {
    fn drop(&mut self) {
        if !self.predefined {
            unsafe { RegCloseKey(self.hkey) };
        }
    }
}
