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

            Ok(())
        }
        _ => Err("".into()),
    }
}
