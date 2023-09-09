use anyhow::{Ok, Result};
use std::cell::RefCell;
use std::io::{Read, Seek};
use std::os::unix::io::AsRawFd;
use std::{fs::File, sync::Arc};

#[cfg(all(feature = "fs", target_os = "linux"))]
mod io_uring;

#[cfg(feature = "fs")]
mod syscall;

/// I/O access method
enum IOMethod {
    Memory,

    #[cfg(feature = "fs")]
    Sync,

    #[cfg(target_os = "linux")]
    IoUring,
}

/// I/O access interface.
pub struct IO {
    io_method: IOMethod,
}

#[cfg(all(feature = "fs", target_os = "linux"))]
impl Default for IO {
    fn default() -> Self {
        IO {
            io_method: IOMethod::IoUring,
        }
    }
}

#[cfg(all(feature = "fs", target_os = "macos"))]
impl Default for IO {
    fn default() -> Self {
        IO {
            io_method: IOMethod::Sync,
        }
    }
}

#[cfg(not(feature = "fs"))]
impl Default for IO {
    fn default() -> Self {
        IO {
            io_method: IOMethod::Memory,
        }
    }
}

impl IO {
    pub fn open(&self, path: &str) -> Result<PageSource> {
        match self.io_method {
            #[cfg(feature = "fs")]
            IOMethod::Sync => {
                let io = Arc::new(syscall::SyscallIO::open(path)?);
                Ok(PageSource { io })
            }
            #[cfg(all(feature = "fs", target_os = "linux"))]
            IOMethod::IoUring => {
                let io = Arc::new(io_uring::IoUring::open(path)?);
                Ok(PageSource { io })
            }
            IOMethod::Memory => {
                todo!();
            }
        }
    }
}

pub struct PageSource {
    io: Arc<dyn PageIO>,
}

impl PageSource {
    pub fn get(&self, page_idx: usize, buf: &mut [u8]) -> Result<()> {
        self.io.get(page_idx, buf)
    }
}

trait PageIO {
    fn get(&self, page_idx: usize, buf: &mut [u8]) -> Result<()>;
}
