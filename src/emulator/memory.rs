use std::collections::BTreeMap;

use crate::error::{Error, Result};

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct MemoryPerms: u8 {
        const READ = 0b001;
        const WRITE = 0b010;
        const EXEC = 0b100;
    }
}

#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub base: u32,
    pub size: u32,
    pub perms: MemoryPerms,
    pub name: String,
}

#[derive(Debug, Clone, Default)]
pub struct MemoryMap {
    regions: BTreeMap<u32, MemoryRegion>,
}

impl MemoryMap {
    pub fn map(&mut self, base: u32, size: u32, perms: MemoryPerms, name: &str) -> Result<()> {
        if size == 0 || base & 0xfff != 0 || size & 0xfff != 0 {
            return Err(Error::InvalidArgument(format!(
                "memory region {name} must be 4 KiB aligned and non-empty"
            )));
        }

        let end = base
            .checked_add(size)
            .ok_or_else(|| Error::InvalidArgument(format!("memory region {name} overflows")))?;

        for region in self.regions.values() {
            let existing_end = region.base + region.size;
            if base < existing_end && end > region.base {
                return Err(Error::InvalidArgument(format!(
                    "memory region {name} overlaps {}",
                    region.name
                )));
            }
        }

        self.regions.insert(
            base,
            MemoryRegion {
                base,
                size,
                perms,
                name: name.to_owned(),
            },
        );
        Ok(())
    }

    pub fn regions(&self) -> impl Iterator<Item = &MemoryRegion> {
        self.regions.values()
    }

    pub fn unmap_exact(&mut self, base: u32, size: u32) -> Option<MemoryRegion> {
        if self
            .regions
            .get(&base)
            .is_some_and(|region| region.size == size)
        {
            self.regions.remove(&base)
        } else {
            None
        }
    }

    pub fn region_containing(&self, address: u32) -> Option<&MemoryRegion> {
        self.regions.values().find(|region| {
            let end = region.base.saturating_add(region.size);
            address >= region.base && address < end
        })
    }

    pub fn contains_range(&self, base: u32, size: u32) -> bool {
        let Some(end) = base.checked_add(size) else {
            return false;
        };
        self.regions.values().any(|region| {
            let region_end = region.base.saturating_add(region.size);
            base >= region.base && end <= region_end
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unmap_exact_removes_only_matching_region() {
        let mut memory = MemoryMap::default();
        memory
            .map(0x1000, 0x2000, MemoryPerms::READ, "first")
            .unwrap();
        memory
            .map(0x4000, 0x1000, MemoryPerms::READ, "second")
            .unwrap();

        assert!(memory.unmap_exact(0x1000, 0x1000).is_none());
        assert!(memory.contains_range(0x1000, 0x2000));
        let removed = memory.unmap_exact(0x1000, 0x2000).unwrap();
        assert_eq!(removed.name, "first");
        assert!(!memory.contains_range(0x1000, 0x2000));
        assert!(memory.contains_range(0x4000, 0x1000));
    }
}
