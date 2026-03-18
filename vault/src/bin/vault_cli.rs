use std::io::Write;

use clap::Parser;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use vault::bips;
use zeroize::Zeroize;

#[derive(clap::Subcommand, Clone, Debug)]
pub enum Command {
    #[command(about = "Initializes the vault by generating passkeys and encrypted structure")]
    Initialize {
        #[arg(long)]
        with_existing_bips: bool,
    },
}

#[derive(clap::Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let mut args = std::env::args();
    // let _ = args.next().ok_or("")?;
    // let verb = args.next().ok_or("")?;

    let args = Cli::parse();
    let mut bips = vec![];
    match args.command {
        Command::Initialize { with_existing_bips } => {
            let config = if !vault::config::config_file_exists().unwrap() {
                vault::config::Config::try_default().unwrap()
            } else {
                vault::config::Config::try_load(
                    vault::config::get_service_config_directory().unwrap(),
                )
                .unwrap()
            };
            // if the user already has a bips39 passphrase
            println!("This function is not complete yet. expect bugs");
            if with_existing_bips {
                let mut phrases = vec![];
                // collect the bips passphrase from secure method
                vault::term::prompt_user_for_bips(&mut phrases)?;
                println!("The program has recieved your bips 39 passcode");
                bips = phrases
            } else {
                println!("This program will generate a bips 39 passphrase. please wait");
                bips = vault::bips::generate_bips()?;
            }
            let qrcode =
                qrcodegen::QrCode::encode_text(&bips.join(" "), qrcodegen::QrCodeEcc::High)?;
            vault::graphics::render_qrcode_to_console(&qrcode);
            println!(
                "Here is your BIPS39 QR code. Make sure you scan this code and save it somewhere.\n Press enter to continue"
            );
            let _ = std::io::stdout().flush().ok();
            std::io::stdin().read_line(&mut String::new())?;
            println!("Your BIPS39 recovery phrase is");
            for (i, phrase) in bips.iter().enumerate() {
                if i % 3 == 0 && i > 0 {
                    println!("{phrase}")
                } else {
                    print!("{phrase} ")
                }
            }
            println!();
            let mut line = String::new();
            println!(
                "Do you want to print a backup copy?\n If Yes, the program will attempt to print to the default printer.\n\
                    Printer selection is not supported at this time.\nPlease type Y or N and press enter"
            );
            std::io::stdin().read_line(&mut line)?;
            line = line.trim().to_string();
            dbg!(&line);
            if line == "Y" {
                println!("the program is attempting to send the job to the requested printer");
                vault::print::win32_print_bip39_using_gdi(&qrcode, &bips)?;
            } else {
                println!("Please make sure you have a backup copy");
            }
            drop(line);
            let mut confirmation = String::new();
            while confirmation != "Y" {
                println!("To confirm that you have your passkey, press Y and then ENTER. If you lose you passkey, \n\
                The app will not be able to decrypt secrets and all vault data will be lost.\n
                If you do not have your passkey, press N an and then enter and run the command again");
                std::io::stdin().read_line(&mut confirmation)?;
                confirmation = confirmation.trim().to_string();
            }
            // we will now 'attempt' to verify the bips
            vault::bips::verify(bips.as_slice()).map_err(|e| e.to_string())?;

            // create the service directory if it doesnt exist
            std::fs::create_dir_all(&config.vault_data_directory).ok();
            std::fs::create_dir_all(&config.vault_config_direcory).ok();
            // write the vault config
            config
                .write(
                    &config
                        .vault_config_direcory
                        .join(vault::config::CONFIG_FILENAME),
                )
                .ok();

            // copy the configuration, files, and dependencies needed into the vault data and config directories
            Ok(())
        }
        _ => Err("".into()),
    }
}
