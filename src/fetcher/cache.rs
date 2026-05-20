use crate::installer::executor::SkillSource;
use crate::utils::{Result, RulesifyError};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

pub fn get_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from(".cache"))
        .join("rulesify")
        .join("archives")
}

pub fn get_cache_key(source: &SkillSource) -> String {
    let input = format!("{}/{}/{}", source.owner, source.repo, source.archive_ref());
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

pub struct ArchiveCache {
    cache_dir: PathBuf,
}

impl ArchiveCache {
    pub fn new() -> Self {
        ArchiveCache {
            cache_dir: get_cache_dir(),
        }
    }

    #[cfg(test)]
    pub(crate) fn with_cache_dir(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }

    pub fn get_cached_path(&self, source: &SkillSource) -> PathBuf {
        let key = get_cache_key(source);
        self.cache_dir.join(key)
    }

    pub fn is_cached(&self, source: &SkillSource) -> bool {
        let cached_path = self.get_cached_path(source);
        cached_path.exists()
    }

    pub async fn download_and_cache(&self, source: &SkillSource) -> Result<PathBuf> {
        let archive_url = format!(
            "https://github.com/{}/{}/archive/{}.tar.gz",
            source.owner,
            source.repo,
            source.archive_ref()
        );

        let response = reqwest::get(&archive_url).await.map_err(|e| {
            RulesifyError::NetworkError(format!("Failed to download archive: {}", e))
        })?;

        if !response.status().is_success() {
            return Err(RulesifyError::NetworkError(format!(
                "GitHub returned status {} for {}",
                response.status(),
                archive_url
            ))
            .into());
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| RulesifyError::NetworkError(format!("Failed to read archive: {}", e)))?;

        let cached_path = self.get_cached_path(source);

        if cached_path.exists() {
            fs::remove_dir_all(&cached_path)?;
        }
        fs::create_dir_all(&cached_path)?;

        let tarball = Cursor::new(bytes.as_ref());

        let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(tarball));
        archive
            .unpack(&cached_path)
            .map_err(|e| RulesifyError::SkillParse(format!("Failed to extract archive: {}", e)))?;

        Ok(cached_path)
    }

    pub async fn get_or_download(&self, source: &SkillSource) -> Result<PathBuf> {
        if self.is_cached(source) {
            return Ok(self.get_cached_path(source));
        }
        self.download_and_cache(source).await
    }

    pub async fn get_extracted_folder(&self, source: &SkillSource) -> Result<PathBuf> {
        let repo_root = self.get_extracted_repo_root(source).await?;
        let folder_path = repo_root.join(&source.folder);

        if !folder_path.exists() {
            return Err(RulesifyError::SkillParse(format!(
                "Folder {} not found in extracted archive",
                source.folder
            ))
            .into());
        }

        Ok(folder_path)
    }

    pub async fn get_extracted_repo_root(&self, source: &SkillSource) -> Result<PathBuf> {
        let cached_path = self.get_or_download(source).await?;
        find_extracted_repo_root(&cached_path)
    }

    pub fn clear(&self) -> Result<()> {
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir)?;
        }
        Ok(())
    }

    pub fn clear_repo(&self, source: &SkillSource) -> Result<()> {
        let cached_path = self.get_cached_path(source);
        if cached_path.exists() {
            fs::remove_dir_all(&cached_path)?;
        }
        Ok(())
    }
}

impl Default for ArchiveCache {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) fn find_extracted_repo_root(cached_path: &Path) -> Result<PathBuf> {
    let roots: Vec<PathBuf> = fs::read_dir(cached_path)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_type()
                .map(|file_type| file_type.is_dir())
                .unwrap_or(false)
        })
        .map(|entry| entry.path())
        .collect();

    match roots.as_slice() {
        [root] => Ok(root.clone()),
        [] => Err(RulesifyError::SkillParse(
            "No extracted repo root found in archive cache".into(),
        )
        .into()),
        _ => {
            let candidates = roots
                .iter()
                .filter_map(|path| path.file_name())
                .map(|name| name.to_string_lossy())
                .collect::<Vec<_>>()
                .join(", ");
            Err(RulesifyError::SkillParse(format!(
                "Expected one extracted repo root in archive cache, found: {}",
                candidates
            ))
            .into())
        }
    }
}
