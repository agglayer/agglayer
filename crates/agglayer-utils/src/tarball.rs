use std::{fs, path::Path};

/// Helper to extract tarball and return path to extracted directory
pub fn extract_tarball(tarball_path: &Path, extract_to: &Path) -> Result<(), eyre::Error> {
    use flate2::read::GzDecoder;
    use tar::Archive;

    fs::create_dir_all(extract_to)?;

    let file = fs::File::open(tarball_path)?;
    let decompressor = GzDecoder::new(file);
    let mut archive = Archive::new(decompressor);

    archive.unpack(extract_to)?;

    Ok(())
}
