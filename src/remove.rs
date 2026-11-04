use std::path::Path;

fn remove_path(path: &Path) -> std::io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::{TempDir, tempdir};

    #[test]
    fn remove_existing_file_deletes_it() {}

    #[test]
    fn remove_missing_file_returns_not_found() {}
}
