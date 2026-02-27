// we need to be able to generate hashes of static files at compile time

use std::{io::Write, path::{Path, PathBuf}};

use sha2::Digest;

fn walk_dir_hash_sha256<D: AsRef<Path>,O: AsRef<Path>>(dir: D, out_dir: O, hashed:&mut std::collections::HashSet<PathBuf>) -> Result<(),Box<dyn std::error::Error>> {
    for entry in std::fs::read_dir(dir.as_ref())? {
        let entry = entry?;
        if entry.metadata()?.is_dir() {
            return walk_dir_hash_sha256(dir.as_ref().join(entry.file_name()), out_dir.as_ref(),hashed);
        }
        let mut f = std::fs::File::open(dir.as_ref().join(entry.file_name()))?;
        let mut hasher = sha2::Sha256::new();
        std::io::copy(&mut f, &mut hasher)?;
        let hash_bytes = hasher.finalize();
        let browserhash = format!("sha256-{:x}", hash_bytes);

        let out_path = out_dir.as_ref().join(entry.file_name().into_string().unwrap() + ".sha256");
        let mut out_file = std::fs::OpenOptions::new().create(true).write(true).truncate(true).open(out_path)?;
        out_file.write(browserhash.as_bytes())?;
        out_file.flush()?;        
    }
    Ok(())

}

fn main() -> Result<(),Box<dyn std::error::Error>> {
    if std::fs::exists("./public/static")? {
        let out_dir = std::env::var("OUT_DIR").map_err(|e| format!("Error reading env OUT_DIR {e}"))?;
        let out_dir = std::path::Path::new(&out_dir).join("browserhash");
        std::fs::create_dir_all(&out_dir).ok();
        walk_dir_hash_sha256(std::path::Path::new("./public/static"), &out_dir, &mut Default::default())?;
    }



    Ok(())

}