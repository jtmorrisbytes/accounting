use std::{io::Write, path::Path};

use zeroize::Zeroize;
/// this function will attempt to scale the qr code within the size specified
/// as the nearest multiple of the size of the code itself
/// output format blue,green,red with unsiged 8 bit integers
// const BLACK_PIXEL: u8 = 0;
// const WHITE_PIXEL: u8 = 0;
pub fn render_qrcode_pix_bgr_u8(qr: &qrcodegen::QrCode, module_size: usize) -> (Vec<u8>,usize) {
    // you cannot have a zero size
    let quiet_zone = 4;
    let total_modules = qr.size() as usize + (quiet_zone * 2);
    
    let pixel_dim = total_modules * module_size;
    let row_bytes = pixel_dim * 3;
    let stride = (row_bytes + 3) & !3; // Align to 4-byte boundary
    // let padding_size = stride - row_bytes;

    // Initialize buffer with white (255) to handle the quiet zone and background
    let mut pixel_data = vec![255_u8; stride * pixel_dim];

    for y in 0..qr.size() {
        for x in 0..qr.size() {
            if qr.get_module(x, y) {
                // Calculate starting pixel including the quiet zone offset
                let start_y = (y as usize + quiet_zone) * module_size;
                let start_x = (x as usize + quiet_zone) * module_size;

                for py in 0..module_size {
                    for px in 0..module_size {
                        let current_y = start_y + py;
                        let current_x = start_x + px;
                        
                        // BMPs are stored bottom-to-top by default. 
                        // To write top-to-bottom, we use a negative height in the header,
                        // which allows us to index the buffer normally.
                        let offset = (current_y * stride) + (current_x * 3);
                        pixel_data[offset] = 0;     // B
                        pixel_data[offset + 1] = 0; // G
                        pixel_data[offset + 2] = 0; // R
                    }
                }
            }
        }
    }
    (pixel_data,pixel_dim)
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



pub fn render_qr_to_svg(qr: &qrcodegen::QrCode) -> String {
    let mut path = String::with_capacity(qr.size() as usize * 15);
    for y in 0..qr.size() {
        for x in 0..qr.size() {
            if qr.get_module(x, y) {
                path.push_str(&format!("M{x},{y}h1v1h-1z "));
            }
        }
    }
    path = path.trim().to_string();
    format!(r#"<path fill="black" d="{path}"/>"#)
}

pub fn render_bips39_phrases_to_html(mut phrases: Vec<String>) -> Result<String,Box<dyn std::error::Error>> {
    // combine the phrase
    let passphrase = phrases.join(" ");
    let qr = qrcodegen::QrCode::encode_text(&passphrase, qrcodegen::QrCodeEcc::High)?;

    let root_viewbox_x = 210;
    let root_viewbox_y = 297;

    let mut qr_svg = render_qr_to_svg(&qr);

    let mut top_offset = ((root_viewbox_y * 5) / 100);
    
    qr_svg = format!("<g transform=\"translate({},{top_offset})\">{qr_svg}<g>",((root_viewbox_x * 50) / 100) - qr.size() / 2 );
    
    top_offset = top_offset + qr.size() as usize;

    // let mut text_start_y = 0;
    let passcode_header = format!("<text y=\"{top_offset}\" font-size=\"6\">Your bip 39 passcode is</text>");

    top_offset = top_offset + 5;

    let mut text_group_contents = String::new();


    let row_height = 16;

    for (i,phrase) in phrases.iter().enumerate() {
        let col = i%4;
        let row = i/4;

        // let col_width = phrase.len() * 5;
        let x = col * (30);
        let y = row * row_height + top_offset;
        // let rect_width = phrase.len() as f32 * 2.0;
        text_group_contents.push_str(&format!("<g transform=\"translate({x},{y})\"> <rect width=\"28\" height=\"10\" fill=\"#f0f0f0\" rx=\"2\" /> <text x=\"5\" y=\"7\" font-family=\"monospace\" font-size=\"5\">{phrase}</text></g>"));
        text_group_contents.push_str("\n");

    }
    phrases.zeroize();
    top_offset = top_offset + row_height * (phrases.len() / 4 ) + 5;

    let message = format!("<text y=\"{top_offset}\" font-size=\"6\" font-family=\"monospace\">Print this file for your records.</text>");
    top_offset = top_offset + 10;
    let message2 = format!("<text y=\"{top_offset}\" font-size=\"4\" font-family=\"monospace\">If you loose this file, there may be no way to recover your data</text>"); 
    
    
    // let text_group = format!("<g transform=translate(0,1) width=\"80%\">{text_group_contents}</g>",top_offset);
    let root_svg = format!("<svg width=\"8.5in\" height=\"11in\" viewBox=\"0 0 210 297\" preserveAspectRatio=\"xMidYMin meet\">{qr_svg}\n{passcode_header}\n{text_group_contents}\n{message}\n{message2}</svg>");

    let html = format!("<!doctype html> <html> <body>{root_svg}</body> </html>");
    // for each passphrase, generate a text node


    Ok(html)
}


pub fn write_qrcode_to_bitmap<P:AsRef<Path>>(path:P,qr: &qrcodegen::QrCode) -> Result<(),Box<dyn std::error::Error>> {
    let (buf,pixel_dims) = self::render_qrcode_pix_bgr_u8(&qr, 2);
    self::write_bitmap_bgr(path, &buf, pixel_dims as i32, pixel_dims as i32)?;

    
    Ok(())
}