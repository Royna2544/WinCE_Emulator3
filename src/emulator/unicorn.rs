use crate::{
    emulator::memory::{MemoryMap, MemoryPerms},
    error::{Error, Result},
};

#[derive(Debug, Clone)]
pub struct UnicornMips {
    memory: MemoryMap,
    entry: Option<u32>,
}

impl UnicornMips {
    pub fn new() -> Result<Self> {
        Ok(Self {
            memory: MemoryMap::default(),
            entry: None,
        })
    }

    pub fn map_region(
        &mut self,
        base: u32,
        size: u32,
        perms: MemoryPerms,
        name: &str,
    ) -> Result<()> {
        self.memory.map(base, size, perms, name)
    }

    pub fn set_entry(&mut self, entry: u32) {
        self.entry = Some(entry);
    }

    pub fn memory(&self) -> &MemoryMap {
        &self.memory
    }

    pub fn run_until_import_trap(&mut self) -> Result<()> {
        #[cfg(feature = "unicorn")]
        {
            return self.run_with_unicorn();
        }

        #[cfg(not(feature = "unicorn"))]
        Err(Error::Backend(
            "built without the `unicorn` feature; core state is ready but CPU execution is disabled"
                .to_owned(),
        ))
    }

    #[cfg(feature = "unicorn")]
    fn run_with_unicorn(&mut self) -> Result<()> {
        use unicorn_engine::{
            Unicorn,
            unicorn_const::{Arch, Mode},
        };

        let _uc = Unicorn::new(Arch::MIPS, Mode::MIPS32 + Mode::LITTLE_ENDIAN)
            .map_err(|err| Error::Backend(format!("{err:?}")))?;

        Err(Error::Backend(
            "Unicorn MIPS backend is linked; PE image mapping and import traps are next".to_owned(),
        ))
    }
}
