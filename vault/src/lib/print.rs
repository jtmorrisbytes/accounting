#[cfg(windows)]
use std::io::Read;

// #[cfg(windows)]
use qrcodegen::QrCode;
#[cfg(windows)]
use windows::Win32::{
    Foundation::{COLORREF, SIZE},
    Graphics::{
        Gdi::{
            BLACKNESS, CLIP_DEFAULT_PRECIS, CreateSolidBrush, DEFAULT_CHARSET, DEFAULT_QUALITY,
            DeleteObject, FW_BOLD, GetDeviceCaps, GetTextExtentPoint32W, HDC, LOGPIXELSX,
            LOGPIXELSY, OUT_DEFAULT_PRECIS, PatBlt, SelectObject, TextOutW,
        },
        Printing::{
            GetJobW, JOB_INFO_1W, JOB_STATUS_COMPLETE, JOB_STATUS_PRINTED, JOB_STATUS_PRINTING,
            JOB_STATUS_SPOOLING, OpenPrinterW, PRINTER_HANDLE,
        },
    },
    Storage::Xps::{EndDoc, EndPage},
};
#[cfg(windows)]
use windows::{
    Win32::Graphics::Gdi::{CreateFontW, FW_NORMAL, HFONT},
    core::PCWSTR,
};
use zeroize::Zeroize;

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

// helper funcs for getjobstatus
fn read_u32_le<R: std::io::Read>(
    r: &mut R,
    field_name: &str,
) -> Result<u32, Box<dyn std::error::Error>> {
    let mut bytes = [0_u8; std::mem::size_of::<u32>()];
    r.read_exact(&mut bytes).map_err(|e| {
        format!(
            "Failed to read {} bytes for field {field_name} because: {e}",
            bytes.len()
        )
    })?;
    let n = u32::from_le_bytes(bytes);
    Ok(n)
}
#[cfg(windows)]
pub fn read_pwstr_ptr_le<R: std::io::Read>(
    r: &mut R,
    field_name: &str,
) -> Result<windows::core::PWSTR, Box<dyn std::error::Error>> {
    let mut bytes = [0_u8; std::mem::size_of::<windows::core::PWSTR>()];
    r.read_exact(&mut bytes).map_err(|e| {
        format!(
            "Failed to read {} bytes for field {field_name} with type PWSTR because: {e}",
            bytes.len()
        )
    })?;

    let address = usize::from_le_bytes(bytes);
    let raw_ptr = address as *mut u16;

    if !raw_ptr.is_aligned() {
        return Err(format!(
            "Attempted to construct raw, unaligned pointer for PWSTR for field {field_name}"
        )
        .into());
    }
    let p = windows::core::PWSTR::from_raw(raw_ptr);
    Ok(p)
}

#[cfg(windows)]
fn win32_get_job_status(
    h_printer: windows::Win32::Graphics::Printing::PRINTER_HANDLE,
    job_id: u32,
) -> Result<(u32, JOB_INFO_1W), Box<dyn std::error::Error>> {
    use windows::Win32::Graphics::Printing::{
        GetJobW, JOB_INFO_1W, JOB_STATUS_ERROR, JOB_STATUS_PRINTED,
    };
    let mut bytes_needed: u32 = 0;

    // 1. Get required buffer size
    unsafe {
        let _ = GetJobW(h_printer, job_id, 1, None, &mut bytes_needed).ok();
    }

    let mut buffer = vec![0u8; bytes_needed as usize];
    // let mut bytes_written: u32 = 0;

    // 2. Fetch actual job info (Level 1)
    let status = unsafe { GetJobW(h_printer, job_id, 1, Some(&mut buffer), &mut bytes_needed).0 };

    if status == 0 {
        return Err("Failed to get job info".into());
    }
    // expect JOB_INFO_1W, we have to build this over each byte

    // for some reason the api surface is dumb and doesnt take a c void. we have to fill out this struct ourselves

    // extract the job id

    let mut job_info = JOB_INFO_1W::default();

    let mut cursor = std::io::Cursor::new(buffer);

    let mut job_id = [0_u8; std::mem::size_of::<u32>()];
    cursor
        .read_exact(&mut job_id)
        .map_err(|e| format!("Failed to read field JobId for JOB_INFO_1W {e}"))?;
    let job_id = u32::from_le_bytes(job_id);
    job_info.JobId = job_id;

    let p = read_pwstr_ptr_le(&mut cursor, "pPrinterName")?;
    job_info.pPrinterName = p;

    let p = read_pwstr_ptr_le(&mut cursor, "pMachineName")?;
    job_info.pMachineName = p;

    let p = read_pwstr_ptr_le(&mut cursor, "pUserName")?;
    job_info.pUserName = p;

    let p = read_pwstr_ptr_le(&mut cursor, "pDocument")?;
    job_info.pDocument = p;

    let p = read_pwstr_ptr_le(&mut cursor, "pStatus")?;
    job_info.pStatus = p;

    let status = read_u32_le(&mut cursor, "Status")?;
    job_info.Status = status;

    let priority = read_u32_le(&mut cursor, "Priority")?;
    job_info.Priority = priority;

    Ok((status, job_info))
}
#[cfg(windows)]
fn win32_create_printer_font(h_dc: HDC, point_size: i32, face_name: &str) -> HFONT {
    // 1. Get the actual vertical DPI of the printer
    let dpi_y = unsafe { GetDeviceCaps(h_dc.into(), LOGPIXELSY) };

    // 2. Calculate height: (Points * DPI) / 72
    // We use a negative value to get the exact point size height
    let height = -((point_size * dpi_y) / 72);

    let face_name = face_name.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
    let p_face_name = PCWSTR(face_name.as_ptr());
    // 3. Create the font
    unsafe {
        CreateFontW(
            height,
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
            p_face_name,
        )
    }
}

// WHY MICROSOFT?? WHY MUST PRINTING BE SO DAMN HARD. Printing requires programming languages sent to printer.
// wont do this. maybe xps works instead?
/// uses GDI to print in XPS document format. requires an XPS compatable printer
#[cfg(windows)]
pub fn win32_print_bip39_using_gdi(
    qr: &QrCode,
    mut bips: &Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    use windows::Win32::Graphics::Gdi::{CreateDCW, DeleteDC};

    use windows::Win32::Storage::Xps::{DOCINFOW, StartDocW, StartPage};
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

    // setting up printing context
    // dots per inch
    let dpi_x = unsafe { GetDeviceCaps(Some(hdc), LOGPIXELSX) };
    // let dpi_y = unsafe { GetDeviceCaps(Some(hdc), LOGPIXELSY) };

    let mut doc_info = DOCINFOW::default();
    // let mut name = "SECURE DOCUMENT".encode_utf16().chain(Some(0)).collect::<Vec<_>>();
    // let name_pwstr = PWSTR(name.as_mut_ptr());
    doc_info.cbSize = std::mem::size_of::<DOCINFOW>().try_into()?;
    doc_info.lpszDocName = windows::core::w!("SECURE DOCUMENT");

    let job_id = unsafe { StartDocW(hdc, &doc_info) };
    if job_id == 0 {
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
    let h_font = win32_create_printer_font(hdc, 12, "Arial");
    if h_font.is_invalid() {
        unsafe {
            let _ = EndPage(hdc);
            let _ = DeleteDC(hdc).ok();
        }
        return Err(windows::core::Error::from_win32().into());
    }

    // render the qr code
    let black_brush = unsafe { CreateSolidBrush(COLORREF(0)) };
    let _old_brush = unsafe { SelectObject(hdc, black_brush.into()) };
    let qr_target_size_inches = 3;
    let total_qr_pixels = (qr_target_size_inches as f64 * dpi_x as f64) as i32;

    let module_size = total_qr_pixels / qr.size();
    let offset_x = (0.5 * dpi_x as f64) as i32;
    let offset_y = (0.5 * dpi_x as f64) as i32;

    for y in 0..qr.size() {
        for x in 0..qr.size() {
            if qr.get_module(x, y) {
                let left = offset_x + (x * module_size);
                let top = offset_y + (y as i32 * module_size);
                unsafe {
                    let _ = PatBlt(hdc, left, top, module_size, module_size, BLACKNESS).ok();
                }
            }
        }
    }
    let _ = unsafe { SelectObject(hdc, _old_brush) };

    let mut top_offset_y = total_qr_pixels + 500;
    // calculate the space that the qr code took up

    // make this font active
    unsafe {
        SelectObject(hdc, h_font.into());
    }

    // write some text
    let header = windows::core::w!("Your Bips 39 passcode is");
    let mut text_extent = SIZE::default();
    let _ = unsafe { GetTextExtentPoint32W(hdc, header.as_wide(), &mut text_extent).ok() };

    // panic!("extent cy {}", text_extent.cy);

    unsafe {
        let _ = TextOutW(hdc, 100, top_offset_y, header.as_wide());
    }
    top_offset_y += text_extent.cy + 100;

    // determine the minimum size of a column. needs to be at least as long as the text
    let mut col_size_x = 0;
    let mut col_size_y = 0;
    for phrase in bips.iter() {
        let wide: Vec<u16> = phrase.encode_utf16().chain(Some(0)).collect();
        let mut extent = SIZE::default();
        unsafe {
            let _ = GetTextExtentPoint32W(hdc, &wide, &mut extent).ok();
        }
        col_size_y = extent.cy.max(col_size_y);
        col_size_x = extent.cx.max(col_size_x);
    }

    let cols = 3;
    for (i, phrase) in bips.iter().enumerate() {
        let wide: Vec<u16> = phrase.encode_utf16().chain(Some(0)).collect();
        let mut text_extent = SIZE::default();
        unsafe { GetTextExtentPoint32W(hdc, &wide, &mut text_extent).ok()? };
        // let pcwstr = PCWSTR(wide.as_ptr());
        let col = i % cols;
        let row = i / cols;
        unsafe {
            let _ = TextOutW(
                hdc,
                col_size_x * col as i32 + 100,
                col_size_y * row as i32 + top_offset_y,
                &wide,
            );
        }
    }
    top_offset_y += col_size_y * cols as i32;

    top_offset_y += top_offset_y + 60;

    unsafe {
        let _ = TextOutW(
            hdc,
            100,
            top_offset_y,
            windows::core::w!("Please save this doc. if you lose, data may not be unrecoverable")
                .as_wide(),
        );
    }

    let _ = unsafe { EndPage(hdc) };
    let _ = unsafe { EndDoc(hdc) };
    // let _ = unsafe { SetJo}
    let _ = unsafe { DeleteObject(h_font.into()) };

    let _ = unsafe { DeleteDC(hdc) };

    // wait for the job to complete
    let mut printer = PRINTER_HANDLE::default();
    unsafe {
        OpenPrinterW(p_default_printer, &mut printer, None)?;
    }
    loop {
        let job_status = win32_get_job_status(printer, job_id as u32)?;
        match job_status.0 {
            JOB_STATUS_PRINTING | JOB_STATUS_SPOOLING => {
                continue;
            }
            JOB_STATUS_COMPLETE => return Ok(()),
            _ => return Ok(()),
        }
    }
}

#[cfg(windows)]
pub mod tests {
    use zeroize::Zeroize;

    use crate::print::win32_print_bip39_using_gdi;

    // #[test]
    // only enable this test if you have a printer
    pub fn test_print_data_from_memory() -> Result<(), Box<dyn std::error::Error>> {
        let bips = crate::bips::generate_bips()?;
        let mut passphrase = bips.join(" ");

        let qr = qrcodegen::QrCode::encode_text(&passphrase, qrcodegen::QrCodeEcc::High)?;
        passphrase.zeroize();

        let name = win32_print_bip39_using_gdi(&qr, &bips)?;
        Ok(())
    }
}
