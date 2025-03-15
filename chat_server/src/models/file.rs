use std::path::{Path, PathBuf};

use super::ChatFile;

use sha1::{Digest, Sha1};

impl ChatFile {
    pub fn new(ws_id: u64, filename: String, data: &[u8]) -> Self {
        let hash = Sha1::digest(data);
        let ext = filename.rsplit('.').next().unwrap_or("txt");
        Self {
            ws_id,
            ext: ext.to_string(),
            hash: hex::encode(hash),
        }
    }

    pub fn url(&self) -> String {
        format!("/files/{}/{}", self.ws_id, self.hash_to_path())
    }

    pub fn path(&self, base_dir: &Path) -> PathBuf {
        base_dir.join(self.hash_to_path())
    }

    // 典型的文件目录的存储方式：
    // 1. 将文件的hash值分成3段，每段3个字符，共9个字符
    // 2. 将这3段字符串作为目录名，将文件的hash值作为文件名
    // 3. 将文件存储在对应的目录中
    // 4. 返回文件的url
    pub fn hash_to_path(&self) -> String {
        let (part1, part2) = self.hash.split_at(3);
        let (part2, part3) = part2.split_at(3);
        format!("{}/{}/{}.{}", part1, part2, part3, self.ext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_to_path() {
        let file = ChatFile::new(1, "test.txt".to_string(), b"hello");
        assert_eq!(
            file.hash_to_path(),
            "aaf/4c6/1ddcc5e8a2dabede0f3b482cd9aea9434d.txt"
        );
    }

    #[test]
    fn test_new_should_work() {
        let file = ChatFile::new(1, "test.txt".to_string(), b"hello");
        assert_eq!(file.ext, "txt");
        assert_eq!(file.hash, "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d");
    }
}
