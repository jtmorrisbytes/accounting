use anyhow::Context;

// pub fn generate_vault_primary_pkcs8_from_entropy(entropy: &[u8;32]) -> Result<Vec<u8>,openssl::error::ErrorStack> {
//     let p = PKey::private_key_from_raw_bytes(entropy.as_slice(),Id::X25519)?;

//     let e = p.private_key_to_pem_pkcs8_passphrase(
//         Cipher::aes_256_cbc(),
//         passphrase
//     )?
//     Ok(e)
// }

// workaround since hkdf interface not yet supported in main openssl
// this function requires  openssl 1.1.1

fn hdkf<PKeyBuf: AsRef<[u8]>, Salt: AsRef<[u8]>, Info: AsRef<[u8]>>(
    pkey: PKeyBuf,
    salt: Salt,
    info: Info,
) -> Result<([u8; 32], usize), anyhow::Error> {
    let salt_ = aws_lc_rs::hkdf::Salt::new(aws_lc_rs::hkdf::HKDF_SHA256, salt.as_ref());
    let prk = salt_.extract(pkey.as_ref());
    let info = info.as_ref();
    let input = &[info];
    let okm = prk
        .expand(input, aws_lc_rs::hkdf::HKDF_SHA256)
        .context("While expanding private key")?;

    let mut out = [0_u8; 32];
    okm.fill(&mut out)
        .context("Output buffer size must match expansion")?;
    Ok((out, out.len()))
}
#[derive(Debug)]
pub struct VaultIntermediateKey {
    hkdf_bytes: [u8; 32],
    salt: [u8; 32],
}

pub(crate) fn gen_hkdf_intermediate_from_256_bits_entropy(
    entropy: &[u8; 32],
    app_name: &str,
    app_version: &str,
    purpose: &str,
) -> Result<VaultIntermediateKey, anyhow::Error> {
    // openssl
    // generate a random salt
    let mut salt_bytes = [0_u8; 32];
    aws_lc_rs::rand::fill(&mut salt_bytes)
        .context("While filling salt bytes for intermediate key")?;
    let key_info = format!("{app_name} | {app_version} | {purpose}");
    let (out, len) = hdkf(entropy, &salt_bytes, key_info.as_bytes())?;
    let key = VaultIntermediateKey {
        hkdf_bytes: out,
        salt: salt_bytes,
    };
    Ok(key)
}

#[test]
pub fn vault_crypto_test_intermediate_key() {
    let mut entropy = [0_u8; 32];
    aws_lc_rs::rand::fill(&mut entropy)
        .context("While filling salt bytes for intermediate key")
        .unwrap();
    let _key =
        gen_hkdf_intermediate_from_256_bits_entropy(&entropy, "VAULT", "0.1.0", "SECRETS").unwrap();
    // dbg!(key);
}
