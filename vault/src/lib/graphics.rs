use std::{io::Write, ops::Neg, path::Path};

use zeroize::Zeroize;
/// this function will attempt to scale the qr code within the size specified
/// as the nearest multiple of the size of the code itself
/// output format blue,green,red with unsiged 8 bit integers
const BLACK_PIXEL: u8 = 0;
const WHITE_PIXEL: u8 = 0;
pub fn render_qrcode_pix_bgr_u8(qr: &qrcodegen::QrCode, mut size: usize) -> Vec<u8> {
    // you cannot have a zero size
    if size == 0 {
        size = 1
    }
    if qr.size() == 0 {
        panic!("Qr code generated a zero size. devide by zero")
    }
    dbg!(qr.size());

    // determine the approx scale

    let scale = (size as f32 / qr.size() as f32).round();

    // this funtion does not support scaling smaller

    let scale = scale.max(1.0) as usize;

    // traverse the module
    let pixel_height = qr.size() as usize * scale;
    let pixel_width = qr.size() as usize * scale;
    let raw_row_bytes = pixel_width * 3;
    let stride = (raw_row_bytes + 3) & !3;
    let mut buffer = vec![255_u8; stride * pixel_height];
    for y in 0..qr.size() {
        for x in 0..qr.size() {
            let module = qr.get_module(x, y);
            if module == false {
                continue;
            }
            for scale_y in 0..scale {
                for scale_x in 0..scale {
                    let current_pixel_y = (y as usize * scale) + scale_y;
                    let current_pixel_x = (x as usize * scale) + scale_x;

                    let offset_start = ((current_pixel_y * stride) + (current_pixel_x * 3)) as usize; 
                    buffer[offset_start] = 0; // blue
                    buffer[offset_start + 1] = 0;
                    buffer[offset_start + 2] = 0;
                }
            }
        }
    }
    dbg!(scale);
    buffer
}

pub fn render_qrcode_to_console(qr: &qrcodegen::QrCode) -> () {
    let size = qr.size();
    let border = 2; // "Quiet Zone" for the scanner

    for y in (-border..size + border).step_by(2) {
        for x in -border..size + border {
            // We check TWO vertical pixels at once to use half-block characters
            let top = qr.get_module(x, y);
            let bottom = qr.get_module(x, y + 1);

            match (top, bottom) {
                (true, true) => print!(" "),   // Both black (empty space)
                (true, false) => print!("▄"),  // Top black, bottom white
                (false, true) => print!("▀"),  // Top white, bottom black
                (false, false) => print!("█"), // Both white (full block)
            }
        }
        println!();
    }
}
/// assumes the data is a valid bitmap file and writes it to the destination specified 
pub fn write_bitmap_bgr<P: AsRef<Path>>(path: P , buffer: &Vec<u8>,bi_width:i32,bi_height:i32) -> std::io::Result<()> {
    let mut output_buf = vec![];
    output_buf.extend_from_slice(b"BM".into_iter().map(|b|b.to_le()).collect::<Vec<_>>().as_slice());
    output_buf.extend_from_slice((54_u32 + buffer.len() as u32).to_le_bytes().as_slice());
    // reserved 1
    output_buf.extend_from_slice(0_u16.to_le_bytes().as_slice());
    // reserved 2
    output_buf.extend_from_slice(0_u16.to_le_bytes().as_slice());
    output_buf.extend_from_slice(54_u32.to_le_bytes().as_slice());


    // BITMAPINFOHEADER
    output_buf.extend_from_slice(40_u32.to_le_bytes().as_slice());
    output_buf.extend_from_slice(bi_width.to_le_bytes().as_slice());
    output_buf.extend_from_slice((-bi_height).to_le_bytes().as_slice());
    // biplanes
    output_buf.extend_from_slice(1_u16.to_le_bytes().as_slice());
    //bi bit count
    output_buf.extend_from_slice(24_u16.to_le_bytes().as_slice());
    // compression
    output_buf.extend_from_slice(0_u32.to_le_bytes().as_slice());
    // the size of the image
    output_buf.extend_from_slice((buffer.len() as u32).to_le_bytes().as_slice());
    // dpi
    output_buf.extend_from_slice(2835_i32.to_le_bytes().as_slice());
    output_buf.extend_from_slice(2835_i32.to_le_bytes().as_slice());
    // color pallete
    output_buf.extend_from_slice(0_u32.to_le_bytes().as_slice());
    // biclrimportant
    output_buf.extend_from_slice(0_u32.to_le_bytes().as_slice());

    output_buf.extend(buffer);

    let mut file = std::fs::OpenOptions::new().create(true).truncate(true).write(true).open(path)?;
    file.write_all(&output_buf)?;
    file.flush()?;
    drop(file);
    std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);

    // secure erase memory for this operation
    output_buf.zeroize();







    Ok(())
}