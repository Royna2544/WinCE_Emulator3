//! Storage-only candidate for final, patched PE image bytes.
//!
//! This module does not publish native mappings. A future `mem_map_ptr` caller
//! must retain the `SealedRunPin` through checked unmap or owner closure.

use std::{alloc::Layout, ptr::NonNull, rc::Rc};

use crate::{
    emulator::memory::MemoryPerms,
    error::{Error, Result},
};

const IMAGE_PAGE_SIZE: usize = 0x1000;

#[derive(Debug)]
struct SealedImageRun {
    bytes: NonNull<u8>,
    len: usize,
    layout: Layout,
}

#[derive(Debug, Clone)]
enum ImageRunBacking {
    Sealed { bytes: Rc<SealedImageRun>, perms: MemoryPerms },
    Mutable(Vec<u8>),
}

#[derive(Debug, Clone)]
struct ImageRun {
    rva: usize,
    backing: ImageRunBacking,
}

impl ImageRun {
    fn len(&self) -> usize {
        match &self.backing {
            ImageRunBacking::Sealed { bytes, .. } => bytes.len,
            ImageRunBacking::Mutable(bytes) => bytes.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SegmentedImageStorage {
    len: usize,
    runs: Vec<ImageRun>,
}

/// Keeps one immutable, page-aligned host allocation alive. The host allocation
/// remains RW for Unicorn, while `perms` applies only to the guest mapping.
#[derive(Debug, Clone)]
pub(crate) struct SealedRunPin {
    rva: u32,
    perms: MemoryPerms,
    backing: Rc<SealedImageRun>,
}

impl SealedRunPin {
    pub(crate) fn rva(&self) -> u32 { self.rva }
    pub(crate) fn len(&self) -> usize { self.backing.len }
    pub(crate) fn guest_perms(&self) -> MemoryPerms { self.perms }
    pub(crate) fn host_ptr(&self) -> *const u8 { self.backing.bytes.as_ptr() }
}

impl SegmentedImageStorage {
    /// Metadata is sorted `(page_rva, guest_perms)` for complete eligible pages.
    pub(crate) fn from_final_bytes(
        _bytes: Vec<u8>,
        _eligible_pages: &[(u32, MemoryPerms)],
    ) -> Result<Self> {
        Err(Error::Unsupported("segmented image storage not yet implemented".to_owned()))
    }

    pub(crate) fn len(&self) -> usize { self.len }

    pub(crate) fn sealed_runs(&self) -> impl Iterator<Item = SealedRunPin> + '_ {
        self.runs.iter().filter_map(|_run| None)
    }

    pub(crate) fn read_at(&self, _rva: u32, _destination: &mut [u8]) -> Result<()> {
        Err(Error::Unsupported("segmented image read not yet implemented".to_owned()))
    }

    pub(crate) fn write_mutable_at(&mut self, _rva: u32, _bytes: &[u8]) -> Result<()> {
        Err(Error::Unsupported("segmented image write not yet implemented".to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_page_image() -> Vec<u8> {
        let mut image = vec![0x41; IMAGE_PAGE_SIZE * 2];
        image[IMAGE_PAGE_SIZE..].fill(0x62);
        image
    }

    fn read_all(image: &SegmentedImageStorage) -> Vec<u8> {
        let mut bytes = vec![0; image.len()];
        image.read_at(0, &mut bytes).unwrap();
        bytes
    }

    #[test]
    fn sealed_read_page_and_mutable_second_page_partition_full_image() {
        let image = SegmentedImageStorage::from_final_bytes(
            two_page_image(),
            &[(0, MemoryPerms::READ | MemoryPerms::EXEC)],
        ).unwrap();
        assert_eq!(image.len(), IMAGE_PAGE_SIZE * 2);
        assert_eq!(image.runs.len(), 2);
        assert!(matches!(image.runs[0].backing, ImageRunBacking::Sealed { .. }));
        assert!(matches!(image.runs[1].backing, ImageRunBacking::Mutable(_)));
        assert_eq!(read_all(&image), two_page_image());
    }

    #[test]
    fn read_crosses_run_boundary() {
        let image = SegmentedImageStorage::from_final_bytes(
            two_page_image(), &[(0, MemoryPerms::READ)],
        ).unwrap();
        let mut boundary = [0; 8];
        image.read_at((IMAGE_PAGE_SIZE - 4) as u32, &mut boundary).unwrap();
        assert_eq!(boundary, [0x41, 0x41, 0x41, 0x41, 0x62, 0x62, 0x62, 0x62]);
    }

    #[test]
    fn clone_shares_sealed_pointer_but_not_mutable_bytes() {
        let original = SegmentedImageStorage::from_final_bytes(
            two_page_image(), &[(0, MemoryPerms::READ | MemoryPerms::EXEC)],
        ).unwrap();
        let mut cloned = original.clone();
        let first = original.sealed_runs().next().unwrap();
        let second = cloned.sealed_runs().next().unwrap();
        assert_eq!(first.host_ptr(), second.host_ptr());
        assert_eq!(first.guest_perms(), MemoryPerms::READ | MemoryPerms::EXEC);
        cloned.write_mutable_at(IMAGE_PAGE_SIZE as u32, &[0xaa]).unwrap();
        assert_eq!(read_all(&original)[IMAGE_PAGE_SIZE], 0x62);
        assert_eq!(read_all(&cloned)[IMAGE_PAGE_SIZE], 0xaa);
    }

    #[test]
    fn sealed_overlap_rejects_whole_write_before_mutating() {
        let mut image = SegmentedImageStorage::from_final_bytes(
            two_page_image(), &[(IMAGE_PAGE_SIZE as u32, MemoryPerms::READ)],
        ).unwrap();
        let before = read_all(&image);
        assert!(matches!(image.write_mutable_at((IMAGE_PAGE_SIZE - 2) as u32, &[1, 2, 3, 4]), Err(Error::AccessDenied(_))));
        assert_eq!(read_all(&image), before);
    }

    #[test]
    fn adjacent_equal_permission_pages_coalesce_into_aligned_host_run() {
        let image = SegmentedImageStorage::from_final_bytes(
            two_page_image(),
            &[(0, MemoryPerms::READ), (IMAGE_PAGE_SIZE as u32, MemoryPerms::READ)],
        ).unwrap();
        let pins: Vec<_> = image.sealed_runs().collect();
        assert_eq!(pins.len(), 1);
        assert_eq!(pins[0].rva(), 0);
        assert_eq!(pins[0].len(), IMAGE_PAGE_SIZE * 2);
        assert_eq!(pins[0].host_ptr() as usize % IMAGE_PAGE_SIZE, 0);
    }

    #[test]
    fn malformed_metadata_fails_closed() {
        let cases = [
            vec![(1, MemoryPerms::READ)],
            vec![(0, MemoryPerms::WRITE)],
            vec![(0, MemoryPerms::EXEC)],
            vec![(0, MemoryPerms::READ), (0, MemoryPerms::READ)],
            vec![(IMAGE_PAGE_SIZE as u32, MemoryPerms::READ), (0, MemoryPerms::READ)],
            vec![((IMAGE_PAGE_SIZE * 2) as u32, MemoryPerms::READ)],
        ];
        for metadata in cases {
            assert!(matches!(SegmentedImageStorage::from_final_bytes(two_page_image(), &metadata), Err(Error::InvalidArgument(_))), "accepted {metadata:?}");
        }
    }

    #[test]
    fn consumed_image_has_no_retained_full_vec_authority() {
        let image = SegmentedImageStorage::from_final_bytes(
            two_page_image(), &[(0, MemoryPerms::READ)],
        ).unwrap();
        let mutable_bytes: usize = image.runs.iter().filter_map(|run| match &run.backing {
            ImageRunBacking::Mutable(bytes) => Some(bytes.len()),
            ImageRunBacking::Sealed { .. } => None,
        }).sum();
        assert_eq!(mutable_bytes, IMAGE_PAGE_SIZE);
        assert_eq!(image.runs.iter().map(ImageRun::len).sum::<usize>(), image.len());
    }
}
