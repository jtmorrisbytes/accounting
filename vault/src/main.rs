use std::io::Write;

use zeroize::Zeroize;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args();
    let _ = args.next().ok_or("")?;
    let verb = args.next().ok_or("")?;
    match verb.as_str() {
        "generate" => {
            println!(
                "This program will generate some entropy in the form of a bips 39 passphrase, then print it using the default printer"
            );
            let mut passphrases = vault::bips::bips_39()?;
            let qrcode = qrcodegen::QrCode::encode_text(&passphrases.join(" "), qrcodegen::QrCodeEcc::High)?;
            // let html = vault::graphics::render_bips39_phrases_to_html(passphrases)?;
            // std::fs::File::create("bips39.html")?.write_all(html.as_bytes())?;
            vault::print::win32_print_bip39_using_gdi(&qr, passphrases);

            Ok(())
        }
        _ => Err("".into()),
    }
}
