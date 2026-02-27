
// pub fn write_be<const N: usize>(writer: &mut std::io::Write)

/// write unsigned 8 bit integer into buffer
pub fn w8(writer: &mut impl std::io::Write, n: u8) -> Result<(), Box<dyn std::error::Error>> {
    let _ = writer.write(&[n])?;
    Ok(())
}
// write unsigned 16 bit integer into writer as big endian
pub fn w16(writer: &mut impl std::io::Write, n: u16) -> Result<(), Box<dyn std::error::Error>> {
    let _ = writer.write_all(&n.to_be_bytes())?;
    Ok(())
}

/// write unsigned 32 bit integer into writer as big endian
pub fn w32(writer: &mut impl std::io::Write, n: u32) -> Result<(), Box<dyn std::error::Error>> {
    let _ = writer.write_all(&n.to_be_bytes())?;
    Ok(())
}
/// write unsigned 32 bit integer into writer as big endian
pub fn w64(writer: &mut impl std::io::Write, n: u64) -> Result<(), Box<dyn std::error::Error>> {
    let _ = writer.write_all(&n.to_be_bytes())?;
    Ok(())
}
pub fn wb(
    writer: &mut impl std::io::Write,
    b: impl AsRef<[u8]>,
) -> Result<(), Box<dyn std::error::Error>> {
    let b: Vec<u8> = b.as_ref().iter().map(|n| n.to_be()).collect();
    writer.write_all(&b)?;
    Ok(())
}
const TPM_RH_NULL: u32 = 0x40000007;

// commands
const TPM_CC_START_AUTH_SESSION: u32 = 0x176;

#[cfg(windows)]
pub mod win32 {
    use std::io::{Cursor};

    use windows::Win32::{
        Foundation::ERROR_INVALID_PARAMETER,
        System::TpmBaseServices::{
            TBS_COMMAND_LOCALITY_ZERO, TBS_COMMAND_PRIORITY_NORMAL,
            TBS_CONTEXT_PARAMS, TPM_DEVICE_INFO, Tbsi_GetDeviceInfo, Tbsip_Context_Close,
            Tbsip_Submit_Command,
        },
    };

    use crate::tpm_2_0::{TPM_CC_START_AUTH_SESSION, TPM_RH_NULL, w16, w32, wb};

    pub struct TbsiContext(*mut std::ffi::c_void);
    impl TbsiContext {
        pub fn close(self) {
            unsafe {
                Tbsip_Context_Close(self.0);
            }
        }
    }
    pub fn win32_tpm_tbs_get_device_info() -> Result<TPM_DEVICE_INFO, windows::core::Error> {
        let mut info = TPM_DEVICE_INFO::default();
        match unsafe {
            Tbsi_GetDeviceInfo(
                std::mem::size_of_val(&info) as u32,
                std::ptr::from_mut::<TPM_DEVICE_INFO>(&mut info).cast(),
            )
        } {
            0 => Ok(info),
            error @ _ => Err(windows::core::Error::new(
                windows::core::HRESULT::from_win32(error),
                "Failed to Call windows::Win32::System::TpmBaseServices::TbsiGetDeviceInfo.",
            )),
        }
    }

    /// opens a tpm context to be able to submit commands to to the TPM.
    /// Only available on windows
    pub fn win32_tpm_tbs_context_create() -> Result<TbsiContext, windows::core::Error> {
        use windows::{
            Win32::{
                Foundation::{
                    TBS_E_BAD_PARAMETER, TBS_E_INTERNAL_ERROR, TBS_E_INVALID_CONTEXT_PARAM,
                    TBS_E_INVALID_OUTPUT_POINTER, TBS_E_SERVICE_DISABLED,
                    TBS_E_SERVICE_NOT_RUNNING, TBS_E_SERVICE_START_PENDING,
                    TBS_E_TOO_MANY_TBS_CONTEXTS, TBS_E_TPM_NOT_FOUND,
                },
                System::TpmBaseServices::{
                    TBS_CONTEXT_VERSION_TWO,
                    TBS_SUCCESS, Tbsi_Context_Create,
                },
            },
            core::HRESULT,
        };

        #[repr(C, packed)]
        struct TbsParams2 {
            version: u32,
            include_tpms: u32,
        }
        let params = TbsParams2 {
            version: TBS_CONTEXT_VERSION_TWO,
            include_tpms: 0b00000000000000000000000000001110_u32,
        };
        // let params = TBS_CONTEXT_PARAMS {
        //     version:TBS_CONTEXT_VERSION_TWO
        // };
        let mut tbs_context = std::ptr::null_mut();
        let tbs_result = unsafe {
            Tbsi_Context_Create(
                std::mem::transmute::<&TbsParams2, &TBS_CONTEXT_PARAMS>(&params),
                &mut tbs_context,
            )
        };
        if tbs_result == TBS_SUCCESS {
            return Ok(TbsiContext(tbs_context));
        }
        match HRESULT::from_win32(tbs_result) {
            TBS_E_BAD_PARAMETER => Err(windows::core::Error::new(
                TBS_E_BAD_PARAMETER,
                "Failed to initialize a context for Win32 \
                TBS Services Because a parameter to TbsiContextCreate was incorrect or invalid",
            )),
            TBS_E_INTERNAL_ERROR => Err(windows::core::Error::new(
                TBS_E_INTERNAL_ERROR,
                "Failed to initialize a context for win32 TBS services because the TBS api experienced an internal error",
            )),
            TBS_E_INVALID_CONTEXT_PARAM => Err(windows::core::Error::new(
                TBS_E_INVALID_CONTEXT_PARAM,
                "Failed to initialize a context for win32 TBS services because the TBS context parameter providedd to TbsiContextCreate is invalid or corrupt",
            )),
            TBS_E_INVALID_OUTPUT_POINTER => Err(windows::core::Error::new(
                TBS_E_INVALID_OUTPUT_POINTER,
                "Failed to create a context to TBS services because the tbs_context output pointer is invalid",
            )),
            TBS_E_SERVICE_DISABLED => Err(windows::core::Error::new(
                TBS_E_SERVICE_DISABLED,
                "Failed to create a context to TBS services because the TBS service is disabled",
            )),
            TBS_E_SERVICE_NOT_RUNNING => Err(windows::core::Error::new(
                TBS_E_SERVICE_NOT_RUNNING,
                "Failed to create a context to TBS services because the TBS service not running",
            )),
            TBS_E_SERVICE_START_PENDING => Err(windows::core::Error::new(
                TBS_E_SERVICE_NOT_RUNNING,
                "Failed to create a context to TBS services because the TBS service not running",
            )),
            TBS_E_TOO_MANY_TBS_CONTEXTS => Err(windows::core::Error::new(
                TBS_E_TOO_MANY_TBS_CONTEXTS,
                "Failed to create a context to TBS services there are too context handles open or none are available",
            )),
            TBS_E_TPM_NOT_FOUND => Err(windows::core::Error::new(
                TBS_E_TPM_NOT_FOUND,
                "Failed to create a context to TBS services there are no tpm modules available that match the requirements",
            )),
            unknown @ _ => Err(windows::core::Error::new(
                unknown,
                "Unable to create a context to TBS services because an unknown error occured",
            )),
        }
    }

    pub fn win32_tbs_tpm2_start_auth_session(
        context: &TbsiContext,
        nonce: &[u8; 32],
    ) -> Result<(), Box<dyn std::error::Error>> {
        if context.0.is_null() {
            return Err(windows::core::Error::new(
                windows::core::HRESULT::from_win32(ERROR_INVALID_PARAMETER.0),
                "A null TbsiContext was passed into win32_tbs_tpm_start_auth_session. Check you havent closed() before calling this function",
            ).into());
        }
        let buffer: Vec<u8> = vec![];
        let mut c = std::io::Cursor::new(buffer);
        w16(&mut c, 0x8001)?;
        w32(&mut c, 0x41)?;

        w32(&mut c, TPM_CC_START_AUTH_SESSION)?;
        // tpskey: salt encryption

        w32(&mut c, super::TPM_RH_NULL)?;
        // bind
        w32(&mut c, TPM_RH_NULL)?;
        // ncallsize (size of nonce)
        w16(&mut c, 32)?;
        // nonce
        wb(&mut c, nonce)?;
        // salt size
        w16(&mut c, 0)?;
        // session type
        w16(&mut c, 0)?;
        // symalg
        w16(&mut c, 0x6)?;
        // keybits
        w16(&mut c, 0x80)?;
        // sym mode
        w16(&mut c, 0x43)?;
        // auth hash
        w16(&mut c, 0xB)?;

        let size = ((c.get_ref().len() - 1) as u32).to_be_bytes();
        (c.get_mut()[2..2 + (std::mem::size_of::<u32>() / std::mem::size_of::<u8>())]
            .copy_from_slice(&size));

        for byte in c.get_ref() {
            print!(" {byte:08b}")
        }
        println!();

        let mut output = [0_u8; 2048];
        let mut lon = output.len() as u32;
        let status = unsafe {
            Tbsip_Submit_Command(
                context.0,
                TBS_COMMAND_LOCALITY_ZERO,
                TBS_COMMAND_PRIORITY_NORMAL,
                c.get_ref(),
                output.as_mut_ptr(),
                &mut lon,
            )
        };
        if status != 0 {
            return Err(windows::core::Error::new(
                windows::core::HRESULT::from_win32(status),
                "tbsip_submit_command failed",
            )
            .into());
        }
        for byte in &output[0..lon as usize] {
            print!(" {byte:08b}")
        }

        Ok(())
    }
    pub fn a(context: &TbsiContext) -> Result<(), Box<dyn std::error::Error>> {
        let b = [0_u8; 22];
        let mut cursor = Cursor::new(b);
        w16(&mut cursor, 0x8001)?;
        let size = (cursor.get_ref().len() - 1) as u32;
        w32(&mut cursor, size)?;
        w32(&mut cursor, 0x017A)?;
        w32(&mut cursor, 0x1)?;
        w32(&mut cursor, 0x80000000)?;
        w32(&mut cursor, 0x8)?;

        let mut output = [0_u8; 2048];
        let mut lon = output.len() as u32;
        let status = unsafe {
            Tbsip_Submit_Command(
                context.0,
                TBS_COMMAND_LOCALITY_ZERO,
                TBS_COMMAND_PRIORITY_NORMAL,
                cursor.get_ref(),
                output.as_mut_ptr(),
                &mut lon,
            )
        };
        if status != 0 {
            return Err(windows::core::Error::new(
                windows::core::HRESULT::from_win32(status),
                "get sessions failed",
            )
            .into());
        }
        for byte in &output[0..lon as usize] {
            print!(" {byte:08b}")
        }
        Ok(())
    }

    #[cfg(test)]
    pub mod win32_tests {
        use windows::Win32::System::TpmBaseServices::{
            TBS_COMMAND_LOCALITY_ZERO, TBS_COMMAND_PRIORITY_LOW, Tbsip_Context_Close,
            Tbsip_Submit_Command,
        };

        use crate::tpm_2_0::win32::win32_tpm_tbs_context_create;

        #[test]
        pub fn tps_test_context_create() -> windows::core::Result<()> {
            super::win32_tpm_tbs_context_create()?;
            Ok(())
        }
        #[test]
        pub fn tps_test_get_device_info() -> Result<(), windows::core::Error> {
            let info = super::win32_tpm_tbs_get_device_info()?;
            // let context = super::win32_tpm_tbs_context_create()?;
            dbg!(info);
            Ok(())
        }
        #[test]
        pub fn tps_test_start_auth_session() -> Result<(), Box<dyn std::error::Error>> {
            let context = super::win32_tpm_tbs_context_create()?;
            super::win32_tbs_tpm2_start_auth_session(&context, &[0_u8; 32])?;
            context.close();
            Ok(())
        }
        #[test]
        pub fn tps_get_sessions() -> Result<(), Box<dyn std::error::Error>> {
            let context = super::win32_tpm_tbs_context_create()?;
            super::win32_tbs_tpm2_start_auth_session(&context, &[0_u8; 32])?;
            context.close();
            Ok(())
        }
    }
}
