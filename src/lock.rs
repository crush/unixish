use windows::Win32::Foundation::ERROR_ALREADY_EXISTS;
use windows::Win32::Foundation::{CloseHandle, GetLastError, HANDLE};
use windows::Win32::System::Threading::CreateMutexW;
use windows::core::PCWSTR;

pub struct Lock {
    handle: HANDLE,
}

impl Lock {
    pub fn take() -> Option<Self> {
        let name: Vec<u16> = "unixish".encode_utf16().chain(std::iter::once(0)).collect();
        let handle = unsafe { CreateMutexW(None, false, PCWSTR(name.as_ptr())).ok()? };
        let code = unsafe { GetLastError() };
        if code == ERROR_ALREADY_EXISTS {
            let _ = unsafe { CloseHandle(handle) };
            return None;
        }
        Some(Self { handle })
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        let _ = unsafe { CloseHandle(self.handle) };
    }
}
