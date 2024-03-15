use super::*;

pub struct Mount {
    pub fs: Box<dyn FileSystem>,
    pub mount_point: Box<str>,
}

impl Mount {
    #[inline]
    pub fn new(fs: Box<dyn FileSystem>, mount_point: Box<str>) -> Self {
        Self { fs, mount_point }
    }

    #[inline]
    fn trim_mount_point<'a>(&self, path: &'a str) -> &'a str {
        path.trim_start_matches(self.mount_point.as_ref())
    }
}

impl FileSystem for Mount {
    #[inline]
    fn read_dir(&self, path: &str) -> Result<Box<dyn Iterator<Item = Metadata> + Send>> {
        self.fs.read_dir(self.trim_mount_point(path))
    }

    #[inline]
    fn open_file(&self, path: &str) -> Result<FileHandle> {
        self.fs.open_file(self.trim_mount_point(path))
    }

    #[inline]
    fn metadata(&self, path: &str) -> Result<Metadata> {
        self.fs.metadata(self.trim_mount_point(path))
    }

    #[inline]
    fn exists(&self, path: &str) -> Result<bool> {
        self.fs.exists(self.trim_mount_point(path))
    }
}

impl core::fmt::Debug for Mount {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Mount")
            .field("mount_point", &self.mount_point)
            .field("fs", &self.fs)
            .finish()
    }
}
