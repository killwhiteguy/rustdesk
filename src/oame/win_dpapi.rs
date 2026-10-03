use std::{ffi::c_void, ptr};

const CRYPTPROTECT_LOCAL_MACHINE: u32 = 0x4;
const CRYPTPROTECT_UI_FORBIDDEN: u32 = 0x1;

#[repr(C)]
struct DataBlob {
    cb_data: u32,
    pb_data: *mut u8,
}

#[link(name = "Crypt32")]
extern "system" {
    fn CryptProtectData(
        p_data_in: *mut DataBlob,
        sz_data_descr: *const u16,
        p_optional_entropy: *mut DataBlob,
        pv_reserved: *mut c_void,
        p_prompt_struct: *mut c_void,
        dw_flags: u32,
        p_data_out: *mut DataBlob,
    ) -> i32;

    fn CryptUnprotectData(
        p_data_in: *mut DataBlob,
        ppsz_data_descr: *mut *mut u16,
        p_optional_entropy: *mut DataBlob,
        pv_reserved: *mut c_void,
        p_prompt_struct: *mut c_void,
        dw_flags: u32,
        p_data_out: *mut DataBlob,
    ) -> i32;
}

#[link(name = "Kernel32")]
extern "system" {
    fn LocalFree(h_mem: *mut c_void) -> *mut c_void;
}

pub fn protect_machine(data: &[u8]) -> Result<Vec<u8>, String> {
    crypt(data, true)
}

pub fn unprotect_machine(data: &[u8]) -> Result<Vec<u8>, String> {
    crypt(data, false)
}

fn crypt(data: &[u8], protect: bool) -> Result<Vec<u8>, String> {
    if data.is_empty() {
        return Err("refusing to DPAPI-protect empty OAME secret".to_owned());
    }
    if data.len() > u32::MAX as usize {
        return Err("OAME secret is too large for Windows DPAPI".to_owned());
    }

    let mut input = DataBlob {
        cb_data: data.len() as u32,
        pb_data: data.as_ptr() as *mut u8,
    };
    let mut output = DataBlob {
        cb_data: 0,
        pb_data: ptr::null_mut(),
    };

    let flags = CRYPTPROTECT_UI_FORBIDDEN | CRYPTPROTECT_LOCAL_MACHINE;
    let ok = unsafe {
        if protect {
            CryptProtectData(
                &mut input,
                ptr::null(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                flags,
                &mut output,
            )
        } else {
            CryptUnprotectData(
                &mut input,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                flags,
                &mut output,
            )
        }
    };

    if ok == 0 {
        return Err(format!(
            "Windows DPAPI {} failed: {}",
            if protect { "protect" } else { "unprotect" },
            std::io::Error::last_os_error()
        ));
    }

    if output.pb_data.is_null() || output.cb_data == 0 {
        if !output.pb_data.is_null() {
            unsafe {
                LocalFree(output.pb_data as *mut c_void);
            }
        }
        return Err("Windows DPAPI returned an empty OAME secret".to_owned());
    }

    let result = unsafe {
        let bytes = std::slice::from_raw_parts(output.pb_data, output.cb_data as usize).to_vec();
        LocalFree(output.pb_data as *mut c_void);
        bytes
    };
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn machine_scope_round_trip() {
        let secret = b"oame-dpapi-test-secret";
        let protected = protect_machine(secret).expect("DPAPI protect");
        assert_ne!(protected, secret);
        let restored = unprotect_machine(&protected).expect("DPAPI unprotect");
        assert_eq!(restored, secret);
    }
}
