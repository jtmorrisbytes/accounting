use std::io::Write;

use zeroize::Zeroize;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args();
    let _ = args.next().ok_or("")?;
    let verb = args.next().ok_or("")?;
    match verb.as_str() {
        "generate" => {
            println!(
                "This program will generate some entropy in the form of a bips 39 passphrase, then prompt you to print it"
            );
            let mut passphrase = vault::bips::bips_39()?.join(" ");
            let qr = qrcodegen::QrCode::encode_text(&passphrase, qrcodegen::QrCodeEcc::High)?;
            passphrase.zeroize();

            let buf = vault::graphics::render_qrcode_pix_bgr_u8(&qr, qr.size() as usize * 2);
            vault::graphics::write_bitmap_bgr("./example.bmp", &buf, qr.size() * 2, qr.size() * 2)?;
            let b: Vec<u8> = buf.iter().map(|b| format!("{b:08b}\n")).fold(Vec::new(),|mut acc: Vec<_>,value: String|{
                acc.extend_from_slice(value.as_bytes());
                acc
            });
            std::fs::File::create("./example.rawqrcode.binary.txt")?.write_all(&b)?;

            Ok(())
        }
        _ => Err("".into()),
    }
}
