use std::env;

#[derive(Debug)]
pub struct NormalisedPath {
    path: std::path::PathBuf,
}

impl std::fmt::Display for NormalisedPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.display())
    }
}

impl NormalisedPath {
    pub fn new(path: String) -> Result<NormalisedPath, Box<dyn std::error::Error>> {
        if path.is_empty() {
            return Err("Path cannot be empty".into());
        }
        let path = if std::path::Path::new(&path).is_absolute() {
            std::path::PathBuf::from(path)
        } else {
            env::current_dir()?.join(path)
        };

        if !path.exists() {
            return Err(format!("Provided path does not exist: {}", path.display()).into());
        }

        let normalised_path = NormalisedPath {
            path: path.canonicalize()?,
        };

        Ok(normalised_path)
    }

    pub fn path(&self) -> &std::path::Path {
        &self.path
    }
}
