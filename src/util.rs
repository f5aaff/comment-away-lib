use std::fs;
use std::io;
use std::path::{Path, PathBuf};
pub fn find_files(path: &Path, result: &mut Vec<PathBuf>) -> io::Result<()> {
    // If the path is a file, add it to the result
    if path.is_file() {
        result.push(path.to_path_buf());
    } else if path.is_dir() {
        // If it's a directory, only crawl the contents, but skip adding directories
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();

            // Recurse into the directory, but don't add directories to the result
            find_files(&entry_path, result)?;
        }
    }

    Ok(())
}
