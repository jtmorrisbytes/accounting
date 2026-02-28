#[cfg(windows)]
use qrcodegen::QrCode;
#[cfg(windows)]
use windows::Win32::{
    Graphics::Gdi::{
        CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DEFAULT_QUALITY, DeleteObject, FONT_CLIP_PRECISION, FONT_OUTPUT_PRECISION, FONT_QUALITY, FW_BOLD, OUT_DEFAULT_PRECIS, SelectObject, TextOutW
    },
    Storage::Xps::{EndDoc, EndPage},
};

use crate::tpm_2_0::win32;

#[cfg(windows)]
pub fn win32_get_default_printer() -> windows::core::Result<String> {
    use windows::Win32::Graphics::Printing::GetDefaultPrinterW;

    // call once to get the size of the underlying buffer
    let mut buffer_size = 0;
    unsafe {
        let _ = GetDefaultPrinterW(None, &mut buffer_size).ok();
    }
    let mut buffer = vec![0_u16; buffer_size as usize];
    let str = windows::core::PWSTR::from_raw(buffer.as_mut_ptr());
    unsafe {
        GetDefaultPrinterW(Some(str), &mut buffer_size).ok()?;
    }
    let s = unsafe { str.to_string()? };
    Ok(s)
}

#[cfg(windows)]
pub fn win32_open_printer(
    name: &str,
) -> windows::core::Result<windows::Win32::Graphics::Printing::PRINTER_HANDLE> {
    use windows::Win32::Graphics::Printing::{OpenPrinterW, PRINTER_HANDLE};
    use windows::core::PCWSTR;
    let wide = name.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
    let pcwstr = PCWSTR::from_raw(wide.as_ptr());

    let mut printer_handle = PRINTER_HANDLE::default();
    unsafe {
        OpenPrinterW(pcwstr, &mut printer_handle, None)?;
    }
    Ok(printer_handle)
}
#[cfg(windows)]
pub fn win32_close_printer(
    printer_handle: windows::Win32::Graphics::Printing::PRINTER_HANDLE,
) -> windows::core::Result<()> {
    use windows::Win32::Graphics::Printing::ClosePrinter;
    unsafe { ClosePrinter(printer_handle) }
}
#[cfg(windows)]
fn wait_for_job(h_printer: windows::Win32::Graphics::Printing::PRINTER_HANDLE, job_id: u32) {
    use windows::Win32::Graphics::Printing::{
        GetJobW, JOB_INFO_1W, JOB_STATUS_ERROR, JOB_STATUS_PRINTED,
    };
    loop {
        let mut bytes_needed: u32 = 0;

        // 1. Get required buffer size
        unsafe {
            let _ = GetJobW(h_printer, job_id, 1, None, &mut bytes_needed).ok();
        }

        if bytes_needed > 0 {
            let mut buffer = vec![0u8; bytes_needed as usize];
            // let mut bytes_written: u32 = 0;

            // 2. Fetch actual job info (Level 1)
            let success =
                unsafe { GetJobW(h_printer, job_id, 1, Some(&mut buffer), &mut bytes_needed).0 };

            if success != 0 {
                let job_info = unsafe { &*(buffer.as_ptr() as *const JOB_INFO_1W) };
                let status = job_info.Status;

                if status & JOB_STATUS_PRINTED != 0 {
                    println!("Print successful!");
                    break;
                } else if status & JOB_STATUS_ERROR != 0 {
                    eprintln!("Printer reported an error.");
                    break;
                }
            }
        }

        // Don't hammer the CPU; wait between polls
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

// WHY MICROSOFT?? WHY MUST PRINTING BE SO DAMN HARD. Printing requires programming languages sent to printer.
// wont do this. maybe xps works instead?
/// uses GDI to print in XPS document format. requires an XPS compatable printer
#[cfg(windows)]
pub fn win32_print_bip39_using_gdi(
    qr: &QrCode,
    bips: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    use windows::Win32::Graphics::Gdi::{CreateDCW, CreateFontW, DeleteDC};
    use windows::Win32::Graphics::Printing::{
        DOC_INFO_1W, EndDocPrinter, EndPagePrinter, StartDocPrinterW, StartPagePrinter,
        WritePrinter,
    };
    use windows::Win32::Storage::Xps::{DOCINFOW, StartDocW, StartPage};
    use windows::core::{HRESULT, HSTRING, PWSTR};
    let default_printer = win32_get_default_printer()?;
    let default_printer = default_printer
        .encode_utf16()
        .chain(Some(0))
        .collect::<Vec<u16>>();
    let p_default_printer = windows::core::PCWSTR::from_raw(default_printer.as_ptr());

    // open a gdi device context instead
    let hdc = unsafe { CreateDCW(None, p_default_printer, None, None) };
    if hdc.is_invalid() {
        return Err(windows::core::Error::from_win32().into());
    }

    let mut doc_info = DOCINFOW::default();
    // let mut name = "SECURE DOCUMENT".encode_utf16().chain(Some(0)).collect::<Vec<_>>();
    // let name_pwstr = PWSTR(name.as_mut_ptr());
    doc_info.cbSize = std::mem::size_of::<DOCINFOW>().try_into()?;
    doc_info.lpszDocName = windows::core::w!("SECURE DOCUMENT");

    let print_job_id = unsafe { StartDocW(hdc, &doc_info) };
    if print_job_id == 0 {
        unsafe {
            let _ = DeleteDC(hdc).ok();
        }
        // windows::core::Error::new(HRESULT::from_nt(windows::Win32::Foundation::E_FAIL.0),"Failed to start the print job").into()
        return Err(windows::core::Error::from_win32().into());
    }
    let status = unsafe { StartPage(hdc) };
    if status < 0 {
        let _ = unsafe { DeleteDC(hdc) }.ok();
        return Err(windows::core::Error::new(
            windows::core::HRESULT::from_nt(status),
            "Call to start page failed",
        )
        .into());
    }
    let h_font = unsafe {
        CreateFontW(
            60,
            0,
            0,
            0,
            FW_BOLD.0 as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            DEFAULT_QUALITY,
            0,
            windows::core::w!("Arial"),
        )
    };
    if h_font.is_invalid() {
        unsafe {
            let _ = EndPage(hdc);
            let _ = DeleteDC(hdc).ok();
        }
        return Err(windows::core::Error::from_win32().into());
    }
    // make this font active
    unsafe {
        SelectObject(hdc, h_font.into());
    }

    // write some text
    unsafe {
        let _ = TextOutW(hdc, 100, 100, windows::core::w!("hello world").as_wide());
    }
    
    let _  = unsafe {
        EndPage(hdc)
    };
    let _  = unsafe {
        EndDoc(hdc)
    };
    let _  = unsafe {
        DeleteObject(h_font.into())
    };

    let _ = unsafe {
        DeleteDC(hdc)
    };
    // wait_for_job(printer_handle, print_job_id);
    // win32_close_printer(printer_handle)?;
    Ok(())
}

#[cfg(windows)]
pub mod tests {
    use zeroize::Zeroize;

    use crate::print::win32_print_bip39_using_gdi;

    #[test]
    pub fn test_print_data_from_memory() -> Result<(), Box<dyn std::error::Error>> {
        let bips = crate::bips::bips_39()?;
        let mut passphrase = bips.join(" ");

        let qr = qrcodegen::QrCode::encode_text(&passphrase, qrcodegen::QrCodeEcc::High)?;
        passphrase.zeroize();

        let name = win32_print_bip39_using_gdi(&qr, bips)?;
        Ok(())
    }
}
