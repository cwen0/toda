use crate::ptrace;

use std::path::Path;

use anyhow::Result;

mod cwd_replacer;
mod fd_replacer;
mod mmap_replacer;

use tracing::error;

pub trait Replacer {
    fn run(&mut self) -> Result<()>;
}

pub struct UnionReplacer {
    replacers: Vec<Box<dyn Replacer>>,
    ptrace_manager: ptrace::PtraceManager,
}

impl UnionReplacer {
    pub fn new() -> UnionReplacer {
        UnionReplacer {
            replacers: Vec::new(),
            ptrace_manager: ptrace::PtraceManager::default(),
        }
    }

    pub fn prepare<P1: AsRef<Path>, P2: AsRef<Path>>(
        &self,
        detect_path: P1,
        new_path: P2,
    ) -> Result<()> {
        let mut replacers: Vec<Box<dyn Replacer>> = Vec::new();

        match FdReplacer::prepare(&detect_path, &new_path, &self.ptrace_manager) {
            Err(err) => error!("Error while preparing fd replacer: {:?}", err),
            Ok(replacer) => replacers.push(Box::new(replacer)),
        }
        match CwdReplacer::prepare(&detect_path, &new_path, &self.ptrace_manager) {
            Err(err) => error!("Error while preparing cwd replacer: {:?}", err),
            Ok(replacer) => replacers.push(Box::new(replacer)),
        }
        match MmapReplacer::prepare(&detect_path, &new_path, &self.ptrace_manager) {
            Err(err) => error!("Error while preparing mmap replacer: {:?}", err),
            Ok(replacer) => replacers.push(Box::new(replacer)),
        }

        Ok(())
    }
}

impl Replacer for UnionReplacer {
    fn run(&mut self) -> Result<()> {
        for replacer in self.replacers.iter_mut() {
            replacer.run()?;
        }

        Ok(())
    }
}

pub use cwd_replacer::CwdReplacer;
pub use fd_replacer::FdReplacer;
pub use mmap_replacer::MmapReplacer;