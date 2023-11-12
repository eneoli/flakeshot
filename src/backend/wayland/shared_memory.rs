use std::fs;
use std::fs::File;
use std::time::SystemTime;

pub fn create_shm_file(prefix: &str, bytes: u64) -> anyhow::Result<File> {
    let name = gen_random_file_name(prefix)?;

    let options = memfd::MemfdOptions::default()
        .allow_sealing(true);

    let memfile = options.create(name)?;

    memfile.as_file().set_len(bytes)?;

    memfile.add_seals(&[
        memfd::FileSeal::SealShrink,
        memfd::FileSeal::SealGrow,
        memfd::FileSeal::SealSeal,
    ])?;

    Ok(memfile.into_file())
}

fn gen_random_file_name(prefix: &str) -> anyhow::Result<String> {
    let mut duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_millis();

    let file_name = prefix.to_string() + duration.to_string().as_str();

    while file_exists(&file_name.as_str()) {
        duration = duration + 1;
    }

    Ok(file_name)
}

fn file_exists(path: &str) -> bool {
    match fs::metadata(path) {
        Ok(_) => true,
        Err(_) => false,
    }
}