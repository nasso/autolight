use std::{
    os::windows::prelude::OsStrExt,
    ffi::OsStr,
    thread::sleep,
    time::Duration,
    ptr::null_mut,
    iter::once
};

use windows::Win32::{
    Foundation::{
        HWND,
        LPARAM,
        WPARAM,
        BOOL, HANDLE, HINSTANCE, CloseHandle
    },
    UI::WindowsAndMessaging::{
        WM_THEMECHANGED,
        EnumWindows,
        WM_SETTINGCHANGE,
        SendMessageTimeoutW,
        SMTO_NORMAL,
        SMTO_NOTIMEOUTIFNOTHUNG, GetWindowThreadProcessId
    }, System::{Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT}, ProcessStatus::{K32GetModuleFileNameExW, K32GetModuleBaseNameW, K32GetProcessImageFileNameW}}
};

use crate::regkey::{
    RegistryKey,
    RegistryPermission
};

fn os_str(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(once(0)).collect()
}

pub fn get_process_name(hwnd: HWND) -> Option<String> {
    unsafe {
        let mut process_id = 0;

        if GetWindowThreadProcessId(hwnd, &mut process_id as *mut u32) == 0 {
            return None;
        }
        
        let Ok(process) = OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
            false,
            process_id
        ) else {
            return None;
        };

        let mut process_name = [0u16; 512];
        let mut length = process_name.len() as u32 - 1;

        let has_image_name = QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_FORMAT(0),
            windows::core::PWSTR(&mut process_name as *mut u16),
            &mut length
        ).as_bool();
        CloseHandle(process);

        if has_image_name {
            String::from_utf16(&process_name).ok()
        } else {
            None
        }
    }
}

unsafe extern "system" fn refresh_window_callback(hwnd: HWND, _: LPARAM) -> BOOL {
    // these processes are known to require refreshes
    let whitelist = vec![
        "explorer.exe"
    ];

    let Some(process_name) = get_process_name(hwnd) else {
        return BOOL(1);
    };

    if whitelist.iter().any(|&w| process_name.contains(w)) {
        SendMessageTimeoutW(
            hwnd,
            WM_SETTINGCHANGE,
            WPARAM(0),
            LPARAM(os_str("ImmersiveColorSet").as_ptr() as isize),
            SMTO_NORMAL | SMTO_NOTIMEOUTIFNOTHUNG,
            200,
            null_mut()
        );
    
        SendMessageTimeoutW(
            hwnd,
            WM_THEMECHANGED,
            WPARAM(0),
            LPARAM(0),
            SMTO_NORMAL | SMTO_NOTIMEOUTIFNOTHUNG,
            200,
            null_mut()
        );
    }
    
    BOOL(1)
}

pub fn refresh_windows() {
    let key = RegistryKey::open_or_create(
        &RegistryKey::HKCU,
        "SOFTWARE\\Microsoft\\Windows\\DWM",
        RegistryPermission::ReadWrite,
    );

    // update accent color as a way to trigger apps that might listen to it
    let accent = key.get_dword("AccentColor");
    key.set_dword("AccentColor", accent + 1);
    sleep(Duration::from_millis(10));
    key.set_dword("AccentColor", accent);

    // refresh the windows
    unsafe {
        EnumWindows(Some(refresh_window_callback), LPARAM(1));
    }
}