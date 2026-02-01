//! File System Abstraction
//!
//! Cross-platform file I/O with async support.

use std::io::{Read, Write, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::PlatformResult;

/// File open mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileMode {
    /// Read only
    Read,
    /// Write only (creates or truncates)
    Write,
    /// Read and write
    ReadWrite,
    /// Append to existing file
    Append,
}

/// File handle for synchronous operations
pub struct FileHandle {
    path: PathBuf,
    file: std::fs::File,
    mode: FileMode,
}

impl FileHandle {
    /// Open a file with the specified mode
    pub fn open(path: impl AsRef<Path>, mode: FileMode) -> PlatformResult<Self> {
        let path = path.as_ref().to_path_buf();
        
        let file = match mode {
            FileMode::Read => std::fs::File::open(&path)?,
            FileMode::Write => std::fs::File::create(&path)?,
            FileMode::ReadWrite => std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&path)?,
            FileMode::Append => std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(&path)?,
        };

        Ok(Self { path, file, mode })
    }

    /// Get the file path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get the file mode
    pub fn mode(&self) -> FileMode {
        self.mode
    }

    /// Read the entire file contents
    pub fn read_all(&mut self) -> PlatformResult<Vec<u8>> {
        let mut buffer = Vec::new();
        self.file.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    /// Read the file as a string
    pub fn read_string(&mut self) -> PlatformResult<String> {
        let mut buffer = String::new();
        self.file.read_to_string(&mut buffer)?;
        Ok(buffer)
    }

    /// Write data to the file
    pub fn write(&mut self, data: &[u8]) -> PlatformResult<usize> {
        Ok(self.file.write(data)?)
    }

    /// Write a string to the file
    pub fn write_string(&mut self, data: &str) -> PlatformResult<usize> {
        self.write(data.as_bytes())
    }

    /// Flush the file buffer
    pub fn flush(&mut self) -> PlatformResult<()> {
        self.file.flush()?;
        Ok(())
    }

    /// Seek to a position in the file
    pub fn seek(&mut self, pos: SeekFrom) -> PlatformResult<u64> {
        Ok(self.file.seek(pos)?)
    }

    /// Get the file size
    pub fn size(&self) -> PlatformResult<u64> {
        Ok(self.file.metadata()?.len())
    }
}

/// Async file handle for non-blocking operations
pub struct AsyncFileHandle {
    path: PathBuf,
}

impl AsyncFileHandle {
    /// Open a file for async operations
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    /// Read the entire file asynchronously
    pub async fn read_all(&self) -> PlatformResult<Vec<u8>> {
        let mut file = tokio::fs::File::open(&self.path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;
        Ok(buffer)
    }

    /// Read the file as a string asynchronously
    pub async fn read_string(&self) -> PlatformResult<String> {
        let mut file = tokio::fs::File::open(&self.path).await?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer).await?;
        Ok(buffer)
    }

    /// Write data to the file asynchronously
    pub async fn write_all(&self, data: &[u8]) -> PlatformResult<()> {
        let mut file = tokio::fs::File::create(&self.path).await?;
        file.write_all(data).await?;
        Ok(())
    }

    /// Write a string to the file asynchronously
    pub async fn write_string(&self, data: &str) -> PlatformResult<()> {
        self.write_all(data.as_bytes()).await
    }

    /// Check if the file exists
    pub async fn exists(&self) -> bool {
        tokio::fs::metadata(&self.path).await.is_ok()
    }
}

/// File system abstraction
pub struct FileSystem {
    /// Base data directory
    data_dir: PathBuf,
    /// Base cache directory
    cache_dir: PathBuf,
    /// Base save directory
    save_dir: PathBuf,
}

impl FileSystem {
    /// Create a new file system with platform-appropriate directories
    pub fn new() -> Self {
        Self {
            data_dir: Self::default_data_dir(),
            cache_dir: Self::default_cache_dir(),
            save_dir: Self::default_save_dir(),
        }
    }

    /// Create a file system with custom directories
    pub fn with_dirs(data_dir: PathBuf, cache_dir: PathBuf, save_dir: PathBuf) -> Self {
        Self {
            data_dir,
            cache_dir,
            save_dir,
        }
    }

    fn default_data_dir() -> PathBuf {
        #[cfg(target_os = "android")]
        {
            PathBuf::from("/data/data/com.odeza.app/files")
        }
        
        #[cfg(target_os = "ios")]
        {
            PathBuf::from("./Documents")
        }
        
        #[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
        {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        }
        
        #[cfg(not(any(
            target_os = "android",
            target_os = "ios",
            target_os = "windows",
            target_os = "linux",
            target_os = "macos"
        )))]
        {
            PathBuf::from(".")
        }
    }

    fn default_cache_dir() -> PathBuf {
        #[cfg(target_os = "android")]
        {
            PathBuf::from("/data/data/com.odeza.app/cache")
        }
        
        #[cfg(target_os = "ios")]
        {
            PathBuf::from("./Caches")
        }
        
        #[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
        {
            let mut dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            dir.push("cache");
            dir
        }
        
        #[cfg(not(any(
            target_os = "android",
            target_os = "ios",
            target_os = "windows",
            target_os = "linux",
            target_os = "macos"
        )))]
        {
            PathBuf::from("./cache")
        }
    }

    fn default_save_dir() -> PathBuf {
        #[cfg(target_os = "android")]
        {
            PathBuf::from("/data/data/com.odeza.app/files/saves")
        }
        
        #[cfg(target_os = "ios")]
        {
            PathBuf::from("./Documents/saves")
        }
        
        #[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
        {
            let mut dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            dir.push("saves");
            dir
        }
        
        #[cfg(not(any(
            target_os = "android",
            target_os = "ios",
            target_os = "windows",
            target_os = "linux",
            target_os = "macos"
        )))]
        {
            PathBuf::from("./saves")
        }
    }

    /// Get the data directory
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    /// Get the cache directory
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Get the save directory
    pub fn save_dir(&self) -> &Path {
        &self.save_dir
    }

    /// Resolve a path relative to the data directory
    pub fn data_path(&self, path: impl AsRef<Path>) -> PathBuf {
        self.data_dir.join(path)
    }

    /// Resolve a path relative to the cache directory
    pub fn cache_path(&self, path: impl AsRef<Path>) -> PathBuf {
        self.cache_dir.join(path)
    }

    /// Resolve a path relative to the save directory
    pub fn save_path(&self, path: impl AsRef<Path>) -> PathBuf {
        self.save_dir.join(path)
    }

    /// Check if a file exists
    pub fn exists(&self, path: impl AsRef<Path>) -> bool {
        path.as_ref().exists()
    }

    /// Check if a path is a directory
    pub fn is_dir(&self, path: impl AsRef<Path>) -> bool {
        path.as_ref().is_dir()
    }

    /// Check if a path is a file
    pub fn is_file(&self, path: impl AsRef<Path>) -> bool {
        path.as_ref().is_file()
    }

    /// Create a directory and all parent directories
    pub fn create_dir_all(&self, path: impl AsRef<Path>) -> PlatformResult<()> {
        std::fs::create_dir_all(path)?;
        Ok(())
    }

    /// Remove a file
    pub fn remove_file(&self, path: impl AsRef<Path>) -> PlatformResult<()> {
        std::fs::remove_file(path)?;
        Ok(())
    }

    /// Remove a directory and all contents
    pub fn remove_dir_all(&self, path: impl AsRef<Path>) -> PlatformResult<()> {
        std::fs::remove_dir_all(path)?;
        Ok(())
    }

    /// Copy a file
    pub fn copy(&self, from: impl AsRef<Path>, to: impl AsRef<Path>) -> PlatformResult<u64> {
        Ok(std::fs::copy(from, to)?)
    }

    /// Rename/move a file
    pub fn rename(&self, from: impl AsRef<Path>, to: impl AsRef<Path>) -> PlatformResult<()> {
        std::fs::rename(from, to)?;
        Ok(())
    }

    /// Read a file's contents
    pub fn read(&self, path: impl AsRef<Path>) -> PlatformResult<Vec<u8>> {
        Ok(std::fs::read(path)?)
    }

    /// Read a file as a string
    pub fn read_string(&self, path: impl AsRef<Path>) -> PlatformResult<String> {
        Ok(std::fs::read_to_string(path)?)
    }

    /// Write data to a file
    pub fn write(&self, path: impl AsRef<Path>, data: &[u8]) -> PlatformResult<()> {
        std::fs::write(path, data)?;
        Ok(())
    }

    /// Write a string to a file
    pub fn write_string(&self, path: impl AsRef<Path>, data: &str) -> PlatformResult<()> {
        self.write(path, data.as_bytes())
    }

    /// List files in a directory
    pub fn list_dir(&self, path: impl AsRef<Path>) -> PlatformResult<Vec<PathBuf>> {
        let mut entries = Vec::new();
        for entry in std::fs::read_dir(path)? {
            entries.push(entry?.path());
        }
        Ok(entries)
    }

    /// Get file metadata
    pub fn metadata(&self, path: impl AsRef<Path>) -> PlatformResult<std::fs::Metadata> {
        Ok(std::fs::metadata(path)?)
    }

    /// Open a file handle
    pub fn open(&self, path: impl AsRef<Path>, mode: FileMode) -> PlatformResult<FileHandle> {
        FileHandle::open(path, mode)
    }

    /// Create an async file handle
    pub fn async_handle(&self, path: impl AsRef<Path>) -> AsyncFileHandle {
        AsyncFileHandle::new(path)
    }
}

impl Default for FileSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Streaming file reader for large files
pub struct StreamingReader {
    file: std::fs::File,
    buffer_size: usize,
}

impl StreamingReader {
    /// Create a new streaming reader
    pub fn new(path: impl AsRef<Path>, buffer_size: usize) -> PlatformResult<Self> {
        let file = std::fs::File::open(path)?;
        Ok(Self {
            file,
            buffer_size,
        })
    }

    /// Read the next chunk
    pub fn read_chunk(&mut self) -> PlatformResult<Option<Vec<u8>>> {
        let mut buffer = vec![0u8; self.buffer_size];
        match self.file.read(&mut buffer)? {
            0 => Ok(None),
            n => {
                buffer.truncate(n);
                Ok(Some(buffer))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_file_system_creation() {
        let fs = FileSystem::new();
        assert!(!fs.data_dir().as_os_str().is_empty());
    }

    #[test]
    fn test_path_resolution() {
        let fs = FileSystem::new();
        let path = fs.data_path("test/file.txt");
        assert!(path.ends_with("test/file.txt"));
    }

    #[test]
    fn test_file_operations() {
        let fs = FileSystem::new();
        let test_dir = std::env::temp_dir().join("odeza_test");
        let _ = fs.create_dir_all(&test_dir);
        
        let test_file = test_dir.join("test.txt");
        
        // Write
        fs.write_string(&test_file, "Hello, Odeza!").unwrap();
        assert!(fs.exists(&test_file));
        
        // Read
        let content = fs.read_string(&test_file).unwrap();
        assert_eq!(content, "Hello, Odeza!");
        
        // Cleanup
        let _ = fs.remove_file(&test_file);
        let _ = fs.remove_dir_all(&test_dir);
    }

    #[test]
    fn test_file_handle() {
        let test_dir = std::env::temp_dir().join("odeza_test_handle");
        let _ = std::fs::create_dir_all(&test_dir);
        let test_file = test_dir.join("handle_test.txt");
        
        // Write
        {
            let mut handle = FileHandle::open(&test_file, FileMode::Write).unwrap();
            handle.write_string("Test content").unwrap();
            handle.flush().unwrap();
        }
        
        // Read
        {
            let mut handle = FileHandle::open(&test_file, FileMode::Read).unwrap();
            let content = handle.read_string().unwrap();
            assert_eq!(content, "Test content");
        }
        
        // Cleanup
        let _ = std::fs::remove_file(&test_file);
        let _ = std::fs::remove_dir_all(&test_dir);
    }
}
