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
        BOOL
    },
    UI::WindowsAndMessaging::{
        WM_THEMECHANGED,
        EnumWindows,
        WM_SETTINGCHANGE,
        SendMessageTimeoutW,
        SMTO_NORMAL,
        SMTO_NOTIMEOUTIFNOTHUNG
    }
};

use crate::regkey::{
    RegistryKey,
    RegistryPermission
};

fn os_str(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(once(0)).collect()
}

unsafe extern "system" fn refresh_window_callback(hwnd: HWND, _: LPARAM) -> BOOL {
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