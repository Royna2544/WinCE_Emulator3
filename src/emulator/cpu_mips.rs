// MIPS-specific emulation support: types, instruction encoding/decoding,
// trampoline patching, register helpers, and ArchHooks implementation.

use crate::{
    error::{Error, Result},
    pe::PeImage,
};
use std::collections::{HashMap, HashSet};
use unicorn_engine::{RegisterMIPS, Unicorn};

use super::arch::ArchHooks;
use super::pe_loader::align_up_4k;

// ── constants ────────────────────────────────────────────────────────────────

pub(crate) const MIPS_NOP: u32 = 0x0000_0000;
pub(crate) const MIPS_JUMP_TABLE_SELECTOR_SEARCH_BACK: u32 = 512;
const IMAGE_SCN_MEM_EXECUTE: u32 = 0x2000_0000;

pub(crate) const MIPS_JMPBUF_RETURN_PC_SLOT: u32 = 0;
pub(crate) const MIPS_JMPBUF_SP_SLOT: u32 = 1;
pub(crate) const MIPS_JMPBUF_FP_SLOT: u32 = 2;
pub(crate) const MIPS_JMPBUF_RA_SLOT: u32 = 3;
pub(crate) const MIPS_JMPBUF_GP_SLOT: u32 = 4;
pub(crate) const MIPS_JMPBUF_S0_SLOT: u32 = 5;

// ── guest register context ────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MipsGuestContext {
    pub regs: [u32; 32],
    pub hi: u32,
    pub lo: u32,
}

impl MipsGuestContext {
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn zero() -> Self {
        Self {
            regs: [0; 32],
            hi: 0,
            lo: 0,
        }
    }

    pub fn set_v0(&mut self, value: u32) {
        self.regs[2] = value;
    }
}

impl Default for MipsGuestContext {
    fn default() -> Self {
        Self::zero()
    }
}

// ── trampoline types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct MipsTrampolinePatchResult {
    pub range: Option<(u32, u32)>,
    pub jumps: Vec<MipsTrampolineJump>,
    pub external_mapped: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct MipsTrampolineJump {
    pub origin: u32,
    pub stub: u32,
    pub byte_len: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct MipsTrampolineJumpIndex {
    jumps_by_stub: Vec<MipsTrampolineJump>,
}

impl MipsTrampolineJumpIndex {
    pub fn new(jumps: &[MipsTrampolineJump]) -> Self {
        let mut jumps_by_stub = jumps.to_vec();
        jumps_by_stub.sort_by_key(|jump| jump.stub);
        Self { jumps_by_stub }
    }

    pub fn jump_for_pc(&self, pc: u32) -> Option<MipsTrampolineJump> {
        let index = self
            .jumps_by_stub
            .partition_point(|trampoline| trampoline.stub <= pc);
        let trampoline = self.jumps_by_stub.get(index.checked_sub(1)?)?;
        let end = trampoline.stub.checked_add(trampoline.byte_len)?;
        (pc < end).then_some(*trampoline)
    }

    pub fn origin_for_pc(&self, pc: u32) -> Option<u32> {
        self.jump_for_pc(pc).map(|trampoline| trampoline.origin)
    }

    pub fn origin_for_stub(&self, stub: u32) -> Option<u32> {
        let index = self
            .jumps_by_stub
            .binary_search_by_key(&stub, |trampoline| trampoline.stub)
            .ok()?;
        Some(self.jumps_by_stub[index].origin)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LiveTrampolineState {
    pub ranges: Vec<(u32, u32)>,
    pub jumps: Vec<MipsTrampolineJump>,
    pages: HashSet<u32>,
    stub_by_origin: HashMap<u32, u32>,
    jump_index: MipsTrampolineJumpIndex,
}

impl LiveTrampolineState {
    pub fn new_with_stub_map(
        ranges: Vec<(u32, u32)>,
        jumps: Vec<MipsTrampolineJump>,
        stub_by_origin: HashMap<u32, u32>,
    ) -> Self {
        let pages = trampoline_pages_for_ranges(&ranges);
        let jump_index = MipsTrampolineJumpIndex::new(&jumps);
        Self {
            ranges,
            jumps,
            pages,
            stub_by_origin,
            jump_index,
        }
    }

    pub fn extend(&mut self, patch: &MipsTrampolinePatchResult) {
        if let Some(range) = patch.range {
            self.ranges.push(range);
        }
        self.jumps.extend(patch.jumps.iter().copied());
        self.pages = trampoline_pages_for_ranges(&self.ranges);
        self.stub_by_origin = trampoline_stub_by_origin(&self.jumps);
        self.jump_index = MipsTrampolineJumpIndex::new(&self.jumps);
    }

    pub fn target_in_pages(&self, target: u32) -> bool {
        target_in_trampoline_pages(target, &self.pages)
    }

    pub fn target_in_ranges(&self, target: u32) -> bool {
        target_in_ranges(target, &self.ranges)
    }

    pub fn stub_for_origin(&self, origin: u32) -> Option<u32> {
        self.stub_by_origin.get(&origin).copied()
    }

    pub fn origin_for_stub(&self, stub: u32) -> Option<u32> {
        self.jump_index.origin_for_stub(stub)
    }

    pub fn origin_for_pc(&self, pc: u32) -> Option<u32> {
        self.jump_index.origin_for_pc(pc)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MipsUnicornPatch {
    pub rva: u32,
    pub pc: u32,
    pub kind: MipsUnicornPatchKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MipsUnicornPatchKind {
    BranchLikely {
        branch: MipsBranchLikely,
        delay_slot: u32,
    },
    Branch {
        branch: MipsBranchLikely,
        delay_slot: u32,
    },
    Jal {
        target: u32,
        delay_slot: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MipsBranchLikely {
    pub rs: u32,
    pub rt: u32,
    pub target: u32,
    pub inverse_branch: MipsBranch,
    pub link: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MipsBranch {
    Beq,
    Bne,
    Blez,
    Bgtz,
    Bltz,
    Bgez,
}

// ── trampoline patching ───────────────────────────────────────────────────────

pub(crate) fn patch_mips_unicorn_trampolines(
    image: &PeImage,
    load_base: u32,
    mapped: &mut Vec<u8>,
    external_stub_base: Option<u32>,
) -> Result<MipsTrampolinePatchResult> {
    let mut patches = Vec::new();
    for section in &image.sections {
        if section.characteristics & IMAGE_SCN_MEM_EXECUTE == 0 {
            continue;
        }
        let section_size = section.virtual_size.max(section.size_of_raw_data);
        let start = section.virtual_address;
        let Some(end) = start.checked_add(section_size) else {
            return Err(Error::InvalidArgument(format!(
                "{} section {} overflows",
                image.path, section.name
            )));
        };
        let mut jump_table_data_ranges =
            mips_halfword_jump_table_ranges(mapped, load_base, start, end, &image.path)?;
        jump_table_data_ranges.extend(mips_byte_jump_table_ranges(
            mapped,
            load_base,
            start,
            end,
            &image.path,
        )?);
        merge_patch_data_ranges(&mut jump_table_data_ranges);
        let mut rva = start;
        while rva.checked_add(8).is_some_and(|next| next <= end) {
            if mips_patch_rva_overlaps_data_ranges(rva, &jump_table_data_ranges) {
                rva = rva
                    .checked_add(4)
                    .ok_or_else(|| Error::InvalidArgument("section scan overflow".to_owned()))?;
                continue;
            }
            let instruction = read_mapped_word(mapped, rva, &image.path)?;
            let delay_slot = read_mapped_word(mapped, rva + 4, &image.path)?;
            if let Some(branch) = decode_mips_branch_likely(instruction) {
                let pc = load_base.wrapping_add(rva);
                patches.push(MipsUnicornPatch {
                    rva,
                    pc,
                    kind: MipsUnicornPatchKind::BranchLikely { branch, delay_slot },
                });
            } else if let Some(branch) = decode_mips_normal_branch(instruction) {
                let pc = load_base.wrapping_add(rva);
                if delay_slot == MIPS_NOP && pc.wrapping_add(branch.target) == pc.wrapping_add(8) {
                    write_mapped_word(mapped, rva, MIPS_NOP, &image.path)?;
                } else if delay_slot != MIPS_NOP || is_unconditional_taken_branch(branch) {
                    patches.push(MipsUnicornPatch {
                        rva,
                        pc,
                        kind: MipsUnicornPatchKind::Branch { branch, delay_slot },
                    });
                }
            } else if let Some(target) =
                decode_mips_jal_target(load_base.wrapping_add(rva), instruction)
            {
                let pc = load_base.wrapping_add(rva);
                patches.push(MipsUnicornPatch {
                    rva,
                    pc,
                    kind: MipsUnicornPatchKind::Jal { target, delay_slot },
                });
            }
            rva = rva
                .checked_add(4)
                .ok_or_else(|| Error::InvalidArgument("section scan overflow".to_owned()))?;
        }
    }

    if patches.is_empty() {
        return Ok(MipsTrampolinePatchResult::default());
    }

    let aligned_len = align_up_4k(mapped.len() as u32)? as usize;
    let mut stub_bytes = Vec::new();
    if external_stub_base.is_none() && mapped.len() < aligned_len {
        mapped.resize(aligned_len, 0);
    }
    let mut stub_rva = if external_stub_base.is_some() {
        0
    } else {
        aligned_len as u32
    };
    let mut trampoline_jumps = Vec::with_capacity(patches.len());
    for patch in patches {
        let stub_pc = external_stub_base
            .unwrap_or(load_base)
            .wrapping_add(stub_rva);
        let stub_words = match patch.kind {
            MipsUnicornPatchKind::BranchLikely { branch, delay_slot } => {
                branch_likely_stub_words(patch.pc, branch, delay_slot, stub_pc)?
            }
            MipsUnicornPatchKind::Branch { branch, delay_slot } => {
                normal_branch_stub_words(patch.pc, branch, delay_slot, stub_pc)?
            }
            MipsUnicornPatchKind::Jal { target, delay_slot } => {
                jal_stub_words(patch.pc, target, delay_slot, stub_pc)?
            }
        };
        write_mapped_word(
            mapped,
            patch.rva,
            encode_mips_lui(26, stub_pc >> 16),
            &image.path,
        )?;
        write_mapped_word(
            mapped,
            patch.rva + 4,
            encode_mips_ori(26, 26, stub_pc & 0xffff),
            &image.path,
        )?;
        let stub_offset = stub_rva as usize;
        let stub_end = stub_offset
            .checked_add(stub_words.len() * 4)
            .ok_or_else(|| Error::InvalidArgument("branch-likely stub overflow".to_owned()))?;
        trampoline_jumps.push(MipsTrampolineJump {
            origin: patch.pc,
            stub: stub_pc,
            byte_len: (stub_words.len() * 4) as u32,
        });
        let target_bytes = if external_stub_base.is_some() {
            &mut stub_bytes
        } else {
            &mut *mapped
        };
        if target_bytes.len() < stub_end {
            target_bytes.resize(stub_end, 0);
        }
        for (index, word) in stub_words.into_iter().enumerate() {
            let offset = stub_offset + index * 4;
            target_bytes[offset..offset + 4].copy_from_slice(&word.to_le_bytes());
        }
        stub_rva = stub_rva
            .checked_add((stub_end - stub_offset) as u32)
            .ok_or_else(|| Error::InvalidArgument("branch-likely stub RVA overflow".to_owned()))?;
    }
    let (range_base, range_size, external_mapped) = if let Some(stub_base) = external_stub_base {
        let final_len = align_up_4k(stub_rva)? as usize;
        if stub_bytes.len() < final_len {
            stub_bytes.resize(final_len, 0);
        }
        (stub_base, final_len as u32, Some(stub_bytes))
    } else {
        let final_len = align_up_4k(stub_rva)? as usize;
        if mapped.len() < final_len {
            mapped.resize(final_len, 0);
        }
        let range_base = load_base.wrapping_add(aligned_len as u32);
        let range_size = final_len
            .checked_sub(aligned_len)
            .and_then(|size| u32::try_from(size).ok())
            .ok_or_else(|| Error::InvalidArgument("branch trampoline range overflow".to_owned()))?;
        (range_base, range_size, None)
    };
    Ok(MipsTrampolinePatchResult {
        range: Some((range_base, range_size)),
        jumps: trampoline_jumps,
        external_mapped,
    })
}

// ── MIPS branch/jump decode ───────────────────────────────────────────────────

pub(crate) fn decode_mips_branch_likely(instruction: u32) -> Option<MipsBranchLikely> {
    let opcode = instruction >> 26;
    let rs = (instruction >> 21) & 0x1f;
    let rt = (instruction >> 16) & 0x1f;
    let imm = instruction as u16 as i16 as i32;
    let target = (4i32.wrapping_add(imm.wrapping_shl(2))) as u32;

    let inverse_branch = match opcode {
        0x14 => MipsBranch::Bne,
        0x15 => MipsBranch::Beq,
        0x16 => MipsBranch::Bgtz,
        0x17 => MipsBranch::Blez,
        0x01 => match rt {
            0x02 => MipsBranch::Bgez,
            0x03 => MipsBranch::Bltz,
            0x12 => MipsBranch::Bgez,
            0x13 => MipsBranch::Bltz,
            _ => return None,
        },
        _ => return None,
    };
    let link = opcode == 0x01 && matches!(rt, 0x12 | 0x13);
    Some(MipsBranchLikely {
        rs,
        rt,
        target,
        inverse_branch,
        link,
    })
}

pub(crate) fn decode_mips_normal_branch(instruction: u32) -> Option<MipsBranchLikely> {
    let opcode = instruction >> 26;
    let rs = (instruction >> 21) & 0x1f;
    let rt = (instruction >> 16) & 0x1f;
    let imm = instruction as u16 as i16 as i32;
    let target = (4i32.wrapping_add(imm.wrapping_shl(2))) as u32;

    let inverse_branch = match opcode {
        0x04 => MipsBranch::Bne,
        0x05 => MipsBranch::Beq,
        0x06 => MipsBranch::Bgtz,
        0x07 => MipsBranch::Blez,
        0x01 => match rt {
            0x00 => MipsBranch::Bgez,
            0x01 => MipsBranch::Bltz,
            _ => return None,
        },
        _ => return None,
    };
    Some(MipsBranchLikely {
        rs,
        rt,
        target,
        inverse_branch,
        link: false,
    })
}

pub(crate) fn is_unconditional_taken_branch(branch: MipsBranchLikely) -> bool {
    branch.rs == 0 && branch.rt == 0 && branch.inverse_branch == MipsBranch::Bne
}

pub(crate) fn decode_mips_jal_target(pc: u32, instruction: u32) -> Option<u32> {
    let opcode = instruction >> 26;
    if opcode != 0x03 {
        return None;
    }
    let index = instruction & 0x03ff_ffff;
    Some((pc.wrapping_add(4) & 0xf000_0000) | (index << 2))
}

pub(crate) fn decode_trampoline_sentinel_target(
    instruction: u32,
    next_instruction: u32,
) -> Option<u32> {
    let opcode = instruction >> 26;
    let rt = (instruction >> 16) & 0x1f;
    if opcode != 0x0f || rt != 26 {
        return None;
    }
    let next_opcode = next_instruction >> 26;
    let next_rs = (next_instruction >> 21) & 0x1f;
    let next_rt = (next_instruction >> 16) & 0x1f;
    if next_opcode != 0x0d || next_rs != 26 || next_rt != 26 {
        return None;
    }
    Some(((instruction & 0xffff) << 16) | (next_instruction & 0xffff))
}

pub(crate) fn decode_trampoline_long_jump_target(
    instruction: u32,
    next_instruction: u32,
    jump_instruction: u32,
    delay_slot: u32,
) -> Option<u32> {
    let register = decode_mips_lui_rt(instruction)?;
    let next_opcode = next_instruction >> 26;
    let next_rs = (next_instruction >> 21) & 0x1f;
    let next_rt = (next_instruction >> 16) & 0x1f;
    if next_opcode != 0x0d || next_rs != register || next_rt != register {
        return None;
    }
    if !is_mips_jr(jump_instruction, register) || delay_slot != MIPS_NOP {
        return None;
    }
    Some(((instruction & 0xffff) << 16) | (next_instruction & 0xffff))
}

pub(crate) fn is_patched_trampoline_jump(
    instruction: u32,
    next_instruction: u32,
    target: u32,
    trampoline_ranges: &[(u32, u32)],
) -> bool {
    let opcode = instruction >> 26;
    opcode == 0x02 && next_instruction == MIPS_NOP && target_in_ranges(target, trampoline_ranges)
}

pub(crate) fn target_in_ranges(target: u32, ranges: &[(u32, u32)]) -> bool {
    ranges.iter().any(|(base, size)| {
        let end = base.saturating_add(*size);
        target >= *base && target < end
    })
}

fn trampoline_pages_for_ranges(ranges: &[(u32, u32)]) -> HashSet<u32> {
    let mut pages = HashSet::new();
    for (base, size) in ranges {
        if *size == 0 {
            continue;
        }
        let first_page = base >> 12;
        let last_page = base.saturating_add(size.saturating_sub(1)) >> 12;
        for page in first_page..=last_page {
            pages.insert(page);
        }
    }
    pages
}

fn target_in_trampoline_pages(target: u32, pages: &HashSet<u32>) -> bool {
    pages.contains(&(target >> 12))
}

pub(crate) fn trampoline_stub_by_origin(jumps: &[MipsTrampolineJump]) -> HashMap<u32, u32> {
    jumps.iter().map(|jump| (jump.origin, jump.stub)).collect()
}

// ── jump table detection ──────────────────────────────────────────────────────

pub(crate) fn mips_halfword_jump_table_ranges(
    mapped: &[u8],
    load_base: u32,
    start: u32,
    end: u32,
    path: &str,
) -> Result<Vec<(u32, u32)>> {
    let mut ranges = Vec::new();
    let mut rva = start;
    while rva.checked_add(32).is_some_and(|next| next <= end) {
        let Some(range) =
            decode_mips_halfword_jump_table_range(mapped, load_base, start, end, rva, path)?
        else {
            rva = rva
                .checked_add(4)
                .ok_or_else(|| Error::InvalidArgument("section scan overflow".to_owned()))?;
            continue;
        };
        ranges.push(range);
        rva = rva
            .checked_add(4)
            .ok_or_else(|| Error::InvalidArgument("section scan overflow".to_owned()))?;
    }
    Ok(ranges)
}

pub(crate) fn mips_byte_jump_table_ranges(
    mapped: &[u8],
    load_base: u32,
    start: u32,
    end: u32,
    path: &str,
) -> Result<Vec<(u32, u32)>> {
    let mut ranges = Vec::new();
    let mut rva = start;
    while rva.checked_add(28).is_some_and(|next| next <= end) {
        let Some(range) =
            decode_mips_byte_jump_table_range(mapped, load_base, start, end, rva, path)?
        else {
            rva = rva
                .checked_add(4)
                .ok_or_else(|| Error::InvalidArgument("section scan overflow".to_owned()))?;
            continue;
        };
        ranges.push(range);
        rva = rva
            .checked_add(4)
            .ok_or_else(|| Error::InvalidArgument("section scan overflow".to_owned()))?;
    }
    Ok(ranges)
}

fn decode_mips_halfword_jump_table_range(
    mapped: &[u8],
    load_base: u32,
    section_start: u32,
    section_end: u32,
    rva: u32,
    path: &str,
) -> Result<Option<(u32, u32)>> {
    let lui = read_mapped_word(mapped, rva, path)?;
    let addiu = read_mapped_word(mapped, rva + 4, path)?;
    let sll = read_mapped_word(mapped, rva + 8, path)?;
    let addu_index = read_mapped_word(mapped, rva + 12, path)?;
    let lh = read_mapped_word(mapped, rva + 16, path)?;
    let addu_target = read_mapped_word(mapped, rva + 20, path)?;
    let jr = read_mapped_word(mapped, rva + 24, path)?;
    let delay_slot = read_mapped_word(mapped, rva + 28, path)?;

    let Some(base_register) = decode_mips_lui_rt(lui) else {
        return Ok(None);
    };
    if !is_mips_addiu_same_register(addiu, base_register) {
        return Ok(None);
    }
    let Some((index_register, selector_register)) = decode_mips_sll_by_one(sll) else {
        return Ok(None);
    };
    if !is_mips_addu(addu_index, index_register, index_register, base_register) {
        return Ok(None);
    }
    if !is_mips_lh_same_register(lh, index_register) {
        return Ok(None);
    }
    if !is_mips_addu(addu_target, base_register, base_register, index_register) {
        return Ok(None);
    }
    if !is_mips_jr(jr, base_register) || delay_slot != MIPS_NOP {
        return Ok(None);
    }

    let table_pc = ((lui & 0xffff) << 16).wrapping_add(addiu as u16 as i16 as i32 as u32);
    let Some(table_rva) = table_pc.checked_sub(load_base) else {
        return Ok(None);
    };
    if table_rva != rva + 32 || table_rva >= section_end {
        return Ok(None);
    }
    let Some(entry_count) = find_mips_halfword_jump_table_entry_count(
        mapped,
        section_start,
        rva,
        selector_register,
        path,
    )?
    else {
        return Ok(None);
    };
    let byte_len = entry_count.saturating_mul(2);
    if byte_len == 0 {
        return Ok(None);
    }
    let Some(table_end) = table_rva.checked_add(byte_len) else {
        return Ok(None);
    };
    if table_end > section_end {
        return Ok(None);
    }
    Ok(Some((table_rva, byte_len)))
}

fn decode_mips_byte_jump_table_range(
    mapped: &[u8],
    load_base: u32,
    section_start: u32,
    section_end: u32,
    rva: u32,
    path: &str,
) -> Result<Option<(u32, u32)>> {
    let lui = read_mapped_word(mapped, rva, path)?;
    let addiu = read_mapped_word(mapped, rva + 4, path)?;
    let addu_index = read_mapped_word(mapped, rva + 8, path)?;
    let lb = read_mapped_word(mapped, rva + 12, path)?;
    let addu_target = read_mapped_word(mapped, rva + 16, path)?;
    let jr = read_mapped_word(mapped, rva + 20, path)?;
    let delay_slot = read_mapped_word(mapped, rva + 24, path)?;

    let Some(base_register) = decode_mips_lui_rt(lui) else {
        return Ok(None);
    };
    if !is_mips_addiu_same_register(addiu, base_register) {
        return Ok(None);
    }
    let Some((index_register, selector_register)) =
        decode_mips_addu_with_base(addu_index, base_register)
    else {
        return Ok(None);
    };
    if !is_mips_lb_same_register(lb, index_register) {
        return Ok(None);
    }
    if !is_mips_addu(addu_target, base_register, base_register, index_register) {
        return Ok(None);
    }
    if !is_mips_jr(jr, base_register) || delay_slot != MIPS_NOP {
        return Ok(None);
    }

    let table_pc = ((lui & 0xffff) << 16).wrapping_add(addiu as u16 as i16 as i32 as u32);
    let Some(table_rva) = table_pc.checked_sub(load_base) else {
        return Ok(None);
    };
    if table_rva != rva + 28 || table_rva >= section_end {
        return Ok(None);
    }
    let Some(entry_count) =
        find_mips_jump_table_entry_count(mapped, section_start, rva, selector_register, path)?
    else {
        return Ok(None);
    };
    if entry_count == 0 {
        return Ok(None);
    }
    let Some(table_end) = table_rva.checked_add(entry_count) else {
        return Ok(None);
    };
    if table_end > section_end {
        return Ok(None);
    }
    Ok(Some((table_rva, entry_count)))
}

fn find_mips_halfword_jump_table_entry_count(
    mapped: &[u8],
    section_start: u32,
    setup_rva: u32,
    selector_register: u32,
    path: &str,
) -> Result<Option<u32>> {
    find_mips_jump_table_entry_count(mapped, section_start, setup_rva, selector_register, path)
}

fn find_mips_jump_table_entry_count(
    mapped: &[u8],
    section_start: u32,
    setup_rva: u32,
    selector_register: u32,
    path: &str,
) -> Result<Option<u32>> {
    let search_start = setup_rva
        .saturating_sub(MIPS_JUMP_TABLE_SELECTOR_SEARCH_BACK)
        .max(section_start);
    let mut cursor = setup_rva;
    while cursor >= search_start + 4 {
        cursor -= 4;
        let instruction = read_mapped_word(mapped, cursor, path)?;
        if instruction >> 26 == 0x0b
            && ((instruction >> 21) & 0x1f) == selector_register
            && (instruction & 0xffff) != 0
        {
            return Ok(Some(instruction & 0xffff));
        }
    }
    Ok(None)
}

/// Sort ranges by start and merge overlapping/adjacent entries so the per-word
/// patcher scan can binary-search instead of scanning every range.
pub(crate) fn merge_patch_data_ranges(ranges: &mut Vec<(u32, u32)>) {
    ranges.sort_unstable_by_key(|(start, _)| *start);
    let mut merged: Vec<(u32, u32)> = Vec::with_capacity(ranges.len());
    for (start, len) in ranges.drain(..) {
        let end = start.saturating_add(len);
        if let Some((last_start, last_len)) = merged.last_mut() {
            let last_end = last_start.saturating_add(*last_len);
            if start <= last_end {
                *last_len = end.max(last_end).saturating_sub(*last_start);
                continue;
            }
        }
        merged.push((start, len));
    }
    *ranges = merged;
}

/// `ranges` must be sorted by start and non-overlapping (the patcher merges
/// them via `merge_patch_data_ranges`); among ranges starting before the probe
/// end, only the last can still overlap, so one binary search suffices.
pub(crate) fn mips_patch_rva_overlaps_data_ranges(rva: u32, ranges: &[(u32, u32)]) -> bool {
    let probe_end = rva.saturating_add(8);
    let index = ranges.partition_point(|(start, _)| *start < probe_end);
    index > 0 && {
        let (start, len) = ranges[index - 1];
        rva < start.saturating_add(len)
    }
}

// ── instruction predicates ────────────────────────────────────────────────────

pub(crate) fn decode_mips_lui_rt(instruction: u32) -> Option<u32> {
    (instruction >> 26 == 0x0f).then_some((instruction >> 16) & 0x1f)
}

pub(crate) fn is_mips_addiu_same_register(instruction: u32, register: u32) -> bool {
    instruction >> 26 == 0x09
        && ((instruction >> 21) & 0x1f) == register
        && ((instruction >> 16) & 0x1f) == register
}

pub(crate) fn decode_mips_sll_by_one(instruction: u32) -> Option<(u32, u32)> {
    let opcode = instruction >> 26;
    let rs = (instruction >> 21) & 0x1f;
    let rt = (instruction >> 16) & 0x1f;
    let rd = (instruction >> 11) & 0x1f;
    let shamt = (instruction >> 6) & 0x1f;
    let funct = instruction & 0x3f;
    (opcode == 0 && rs == 0 && shamt == 1 && funct == 0).then_some((rd, rt))
}

pub(crate) fn decode_mips_addu_with_base(
    instruction: u32,
    base_register: u32,
) -> Option<(u32, u32)> {
    if instruction >> 26 != 0 || (instruction & 0x3f) != 0x21 {
        return None;
    }
    let rs = (instruction >> 21) & 0x1f;
    let rt = (instruction >> 16) & 0x1f;
    let rd = (instruction >> 11) & 0x1f;
    if rt == base_register {
        Some((rd, rs))
    } else if rs == base_register {
        Some((rd, rt))
    } else {
        None
    }
}

pub(crate) fn is_mips_addu(instruction: u32, rd: u32, rs: u32, rt: u32) -> bool {
    instruction >> 26 == 0
        && ((instruction >> 21) & 0x1f) == rs
        && ((instruction >> 16) & 0x1f) == rt
        && ((instruction >> 11) & 0x1f) == rd
        && (instruction & 0x3f) == 0x21
}

pub(crate) fn is_mips_lh_same_register(instruction: u32, register: u32) -> bool {
    instruction >> 26 == 0x21
        && ((instruction >> 21) & 0x1f) == register
        && ((instruction >> 16) & 0x1f) == register
        && (instruction & 0xffff) == 0
}

pub(crate) fn is_mips_lb_same_register(instruction: u32, register: u32) -> bool {
    instruction >> 26 == 0x20
        && ((instruction >> 21) & 0x1f) == register
        && ((instruction >> 16) & 0x1f) == register
        && (instruction & 0xffff) == 0
}

pub(crate) fn is_mips_jr(instruction: u32, register: u32) -> bool {
    instruction >> 26 == 0
        && ((instruction >> 21) & 0x1f) == register
        && (instruction & 0x001f_ffff) == 0x08
}

// ── stub generation ───────────────────────────────────────────────────────────

pub(crate) fn jal_stub_words(
    pc: u32,
    target: u32,
    delay_slot: u32,
    stub_pc: u32,
) -> Result<Vec<u32>> {
    let link_address = pc.wrapping_add(8);
    let mut words = vec![
        encode_mips_lui(31, link_address >> 16),
        encode_mips_ori(31, 31, link_address & 0xffff),
        delay_slot,
    ];
    append_mips_jump_sequence(&mut words, stub_pc.wrapping_add(12), target)?;
    Ok(words)
}

pub(crate) fn branch_likely_stub_words(
    pc: u32,
    mut branch: MipsBranchLikely,
    delay_slot: u32,
    stub_pc: u32,
) -> Result<Vec<u32>> {
    branch.target = pc.wrapping_add(branch.target);
    let fallthrough = pc.wrapping_add(8);
    let prefix_len = if branch.link { 4 } else { 2 };
    let true_jump_pc = stub_pc.wrapping_add((prefix_len + 1) * 4);
    let true_jump_len = mips_jump_sequence_len(true_jump_pc, branch.target)?;
    let false_path_pc = stub_pc.wrapping_add((prefix_len + 1 + true_jump_len as u32) * 4);

    let mut words = vec![
        encode_mips_cond_branch(
            branch.inverse_branch,
            branch.rs,
            branch.rt,
            stub_pc,
            false_path_pc,
        )?,
        MIPS_NOP,
    ];
    if branch.link {
        let link_address = pc.wrapping_add(8);
        words.push(encode_mips_lui(31, link_address >> 16));
        words.push(encode_mips_ori(31, 31, link_address & 0xffff));
    }
    words.push(delay_slot);
    append_mips_jump_sequence(&mut words, true_jump_pc, branch.target)?;
    append_mips_jump_sequence(&mut words, false_path_pc, fallthrough)?;
    Ok(words)
}

pub(crate) fn normal_branch_stub_words(
    pc: u32,
    mut branch: MipsBranchLikely,
    delay_slot: u32,
    stub_pc: u32,
) -> Result<Vec<u32>> {
    branch.target = pc.wrapping_add(branch.target);
    let fallthrough = pc.wrapping_add(8);
    let true_jump_pc = stub_pc.wrapping_add(12);
    let true_jump_len = mips_jump_sequence_len(true_jump_pc, branch.target)?;
    let false_path_pc = stub_pc.wrapping_add((3 + true_jump_len as u32) * 4);

    let mut words = vec![
        encode_mips_cond_branch(
            branch.inverse_branch,
            branch.rs,
            branch.rt,
            stub_pc,
            false_path_pc,
        )?,
        MIPS_NOP,
        delay_slot,
    ];
    append_mips_jump_sequence(&mut words, true_jump_pc, branch.target)?;
    words.push(delay_slot);
    append_mips_jump_sequence(&mut words, false_path_pc.wrapping_add(4), fallthrough)?;
    Ok(words)
}

pub(crate) fn encode_mips_cond_branch(
    branch: MipsBranch,
    rs: u32,
    rt: u32,
    pc: u32,
    target: u32,
) -> Result<u32> {
    let offset = branch_offset(pc, target)?;
    let instruction = match branch {
        MipsBranch::Beq => (0x04 << 26) | (rs << 21) | (rt << 16),
        MipsBranch::Bne => (0x05 << 26) | (rs << 21) | (rt << 16),
        MipsBranch::Blez => (0x06 << 26) | (rs << 21),
        MipsBranch::Bgtz => (0x07 << 26) | (rs << 21),
        MipsBranch::Bltz => (0x01 << 26) | (rs << 21),
        MipsBranch::Bgez => (0x01 << 26) | (rs << 21) | (0x01 << 16),
    };
    Ok(instruction | u32::from(offset as u16))
}

pub(crate) fn branch_offset(pc: u32, target: u32) -> Result<i16> {
    let delta = target as i64 - pc.wrapping_add(4) as i64;
    if delta % 4 != 0 {
        return Err(Error::InvalidArgument(format!(
            "unaligned MIPS branch target 0x{target:08x}"
        )));
    }
    let offset = delta / 4;
    i16::try_from(offset).map_err(|_| {
        Error::InvalidArgument(format!(
            "MIPS branch target 0x{target:08x} is out of trampoline range from 0x{pc:08x}"
        ))
    })
}

pub(crate) fn encode_mips_jump(pc: u32, target: u32) -> Result<u32> {
    if pc.wrapping_add(4) & 0xf000_0000 != target & 0xf000_0000 {
        return Err(Error::InvalidArgument(format!(
            "MIPS jump target 0x{target:08x} is outside direct jump region from 0x{pc:08x}"
        )));
    }
    Ok((0x02 << 26) | ((target >> 2) & 0x03ff_ffff))
}

pub(crate) fn mips_jump_sequence_len(pc: u32, target: u32) -> Result<usize> {
    if pc.wrapping_add(4) & 0xf000_0000 == target & 0xf000_0000 {
        Ok(2)
    } else {
        Ok(4)
    }
}

pub(crate) fn append_mips_jump_sequence(words: &mut Vec<u32>, pc: u32, target: u32) -> Result<()> {
    if mips_jump_sequence_len(pc, target)? == 2 {
        words.push(encode_mips_jump(pc, target)?);
        words.push(MIPS_NOP);
    } else {
        words.push(encode_mips_lui(26, target >> 16));
        words.push(encode_mips_ori(26, 26, target & 0xffff));
        words.push(encode_mips_jr(26));
        words.push(MIPS_NOP);
    }
    Ok(())
}

pub(crate) fn encode_mips_lui(rt: u32, imm: u32) -> u32 {
    (0x0f << 26) | (rt << 16) | (imm & 0xffff)
}

pub(crate) fn encode_mips_ori(rt: u32, rs: u32, imm: u32) -> u32 {
    (0x0d << 26) | (rs << 21) | (rt << 16) | (imm & 0xffff)
}

pub(crate) fn encode_mips_jr(rs: u32) -> u32 {
    (rs << 21) | 0x08
}

// ── mapped image word helpers (used by patch_mips_unicorn_trampolines) ────────

pub(crate) fn read_mapped_word(mapped: &[u8], rva: u32, path: &str) -> Result<u32> {
    let offset = rva as usize;
    let end = offset
        .checked_add(4)
        .ok_or_else(|| Error::InvalidArgument(format!("{path} mapped read overflows")))?;
    let bytes = mapped.get(offset..end).ok_or_else(|| {
        Error::InvalidArgument(format!(
            "{path} mapped read RVA 0x{rva:08x} is outside image"
        ))
    })?;
    Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
}

pub(crate) fn write_mapped_word(mapped: &mut [u8], rva: u32, value: u32, path: &str) -> Result<()> {
    let offset = rva as usize;
    let end = offset
        .checked_add(4)
        .ok_or_else(|| Error::InvalidArgument(format!("{path} mapped write overflows")))?;
    let bytes = mapped.get_mut(offset..end).ok_or_else(|| {
        Error::InvalidArgument(format!(
            "{path} mapped write RVA 0x{rva:08x} is outside image"
        ))
    })?;
    bytes.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

// ── GPR register helpers ──────────────────────────────────────────────────────

pub(crate) fn read_mips_reg<D>(uc: &Unicorn<'_, D>, register: RegisterMIPS) -> u32 {
    uc.reg_read(register).unwrap_or(0) as u32
}

pub(crate) fn read_mips_import_args<D>(uc: &Unicorn<'_, D>, count: usize) -> Vec<u32> {
    let mut args = vec![
        read_mips_reg(uc, RegisterMIPS::A0),
        read_mips_reg(uc, RegisterMIPS::A1),
        read_mips_reg(uc, RegisterMIPS::A2),
        read_mips_reg(uc, RegisterMIPS::A3),
    ];
    let sp = read_mips_reg(uc, RegisterMIPS::SP);
    for i in 0..count.saturating_sub(4) {
        let offset = 16 + i as u32 * 4;
        let value = sp
            .checked_add(offset)
            .and_then(|addr| {
                let mut bytes = [0u8; 4];
                uc.mem_read(u64::from(addr), &mut bytes)
                    .ok()
                    .map(|()| u32::from_le_bytes(bytes))
            })
            .unwrap_or(0);
        args.push(value);
    }
    args
}

pub(crate) fn read_mips_gpr<D>(uc: &Unicorn<'_, D>, register: u32) -> Option<u32> {
    match register {
        0 => Some(0),
        1 => Some(read_mips_reg(uc, RegisterMIPS::AT)),
        2 => Some(read_mips_reg(uc, RegisterMIPS::V0)),
        3 => Some(read_mips_reg(uc, RegisterMIPS::V1)),
        4 => Some(read_mips_reg(uc, RegisterMIPS::A0)),
        5 => Some(read_mips_reg(uc, RegisterMIPS::A1)),
        6 => Some(read_mips_reg(uc, RegisterMIPS::A2)),
        7 => Some(read_mips_reg(uc, RegisterMIPS::A3)),
        8 => Some(read_mips_reg(uc, RegisterMIPS::T0)),
        9 => Some(read_mips_reg(uc, RegisterMIPS::T1)),
        10 => Some(read_mips_reg(uc, RegisterMIPS::T2)),
        11 => Some(read_mips_reg(uc, RegisterMIPS::T3)),
        12 => Some(read_mips_reg(uc, RegisterMIPS::T4)),
        13 => Some(read_mips_reg(uc, RegisterMIPS::T5)),
        14 => Some(read_mips_reg(uc, RegisterMIPS::T6)),
        15 => Some(read_mips_reg(uc, RegisterMIPS::T7)),
        16 => Some(read_mips_reg(uc, RegisterMIPS::S0)),
        17 => Some(read_mips_reg(uc, RegisterMIPS::S1)),
        18 => Some(read_mips_reg(uc, RegisterMIPS::S2)),
        19 => Some(read_mips_reg(uc, RegisterMIPS::S3)),
        20 => Some(read_mips_reg(uc, RegisterMIPS::S4)),
        21 => Some(read_mips_reg(uc, RegisterMIPS::S5)),
        22 => Some(read_mips_reg(uc, RegisterMIPS::S6)),
        23 => Some(read_mips_reg(uc, RegisterMIPS::S7)),
        24 => Some(read_mips_reg(uc, RegisterMIPS::T8)),
        25 => Some(read_mips_reg(uc, RegisterMIPS::T9)),
        28 => Some(read_mips_reg(uc, RegisterMIPS::GP)),
        29 => Some(read_mips_reg(uc, RegisterMIPS::SP)),
        30 => Some(read_mips_reg(uc, RegisterMIPS::FP)),
        31 => Some(read_mips_reg(uc, RegisterMIPS::RA)),
        _ => None,
    }
}

pub(crate) fn write_mips_gpr<D>(uc: &mut Unicorn<'_, D>, register: u32, value: u32) -> Option<()> {
    let reg = match register {
        0 => return Some(()),
        1 => RegisterMIPS::AT,
        2 => RegisterMIPS::V0,
        3 => RegisterMIPS::V1,
        4 => RegisterMIPS::A0,
        5 => RegisterMIPS::A1,
        6 => RegisterMIPS::A2,
        7 => RegisterMIPS::A3,
        8 => RegisterMIPS::T0,
        9 => RegisterMIPS::T1,
        10 => RegisterMIPS::T2,
        11 => RegisterMIPS::T3,
        12 => RegisterMIPS::T4,
        13 => RegisterMIPS::T5,
        14 => RegisterMIPS::T6,
        15 => RegisterMIPS::T7,
        16 => RegisterMIPS::S0,
        17 => RegisterMIPS::S1,
        18 => RegisterMIPS::S2,
        19 => RegisterMIPS::S3,
        20 => RegisterMIPS::S4,
        21 => RegisterMIPS::S5,
        22 => RegisterMIPS::S6,
        23 => RegisterMIPS::S7,
        24 => RegisterMIPS::T8,
        25 => RegisterMIPS::T9,
        28 => RegisterMIPS::GP,
        29 => RegisterMIPS::SP,
        30 => RegisterMIPS::FP,
        31 => RegisterMIPS::RA,
        _ => return None,
    };
    uc.reg_write(reg, u64::from(value)).ok()
}

pub(crate) fn mips_gpr_name(register: u32) -> &'static str {
    match register {
        0 => "zero",
        1 => "at",
        2 => "v0",
        3 => "v1",
        4 => "a0",
        5 => "a1",
        6 => "a2",
        7 => "a3",
        8 => "t0",
        9 => "t1",
        10 => "t2",
        11 => "t3",
        12 => "t4",
        13 => "t5",
        14 => "t6",
        15 => "t7",
        16 => "s0",
        17 => "s1",
        18 => "s2",
        19 => "s3",
        20 => "s4",
        21 => "s5",
        22 => "s6",
        23 => "s7",
        24 => "t8",
        25 => "t9",
        26 => "k0",
        27 => "k1",
        28 => "gp",
        29 => "sp",
        30 => "fp",
        31 => "ra",
        _ => "?",
    }
}

// ── context capture / restore ─────────────────────────────────────────────────

pub(crate) fn capture_mips_gprs<D>(uc: &Unicorn<'_, D>) -> MipsGuestContext {
    let mut regs = [0; 32];
    for register in 1..32 {
        regs[register as usize] = read_mips_gpr(uc, register).unwrap_or(0);
    }
    MipsGuestContext {
        regs,
        hi: uc.reg_read(RegisterMIPS::HI).unwrap_or(0) as u32,
        lo: uc.reg_read(RegisterMIPS::LO).unwrap_or(0) as u32,
    }
}

pub(crate) fn restore_mips_gprs<D>(uc: &mut Unicorn<'_, D>, context: &MipsGuestContext) {
    for register in 1..32 {
        let _ = write_mips_gpr(uc, register, context.regs[register as usize]);
    }
    let _ = uc.reg_write(RegisterMIPS::HI, u64::from(context.hi));
    let _ = uc.reg_write(RegisterMIPS::LO, u64::from(context.lo));
}

// ── delay-slot and control-flow analysis ─────────────────────────────────────

/// Returns true if `pc` is a branch-delay slot given a function to read guest words.
pub(crate) fn is_mips_delay_slot_pc(
    read_u32: impl Fn(u32) -> Option<u32>,
    previous_pc: Option<u32>,
    pc: u32,
) -> bool {
    let Some(previous_pc) = previous_pc else {
        return false;
    };
    if previous_pc.wrapping_add(4) != pc {
        return false;
    }
    read_u32(previous_pc).is_some_and(is_mips_control_transfer_instruction)
}

pub(crate) fn is_mips_control_transfer_instruction(instruction: u32) -> bool {
    let opcode = instruction >> 26;
    match opcode {
        0x00 => matches!(instruction & 0x3f, 0x08 | 0x09),
        0x01 => matches!(
            (instruction >> 16) & 0x1f,
            0x00 | 0x01 | 0x02 | 0x03 | 0x10 | 0x11 | 0x12 | 0x13
        ),
        0x02 | 0x03 | 0x04 | 0x05 | 0x06 | 0x07 | 0x14 | 0x15 | 0x16 | 0x17 => true,
        0x10..=0x13 => ((instruction >> 21) & 0x1f) == 0x08,
        _ => false,
    }
}

pub(crate) fn mips_trampoline_origin_for_pc(
    pc: u32,
    trampoline_jumps: &[MipsTrampolineJump],
) -> Option<u32> {
    trampoline_jumps.iter().find_map(|trampoline| {
        let end = trampoline.stub.checked_add(trampoline.byte_len)?;
        (pc >= trampoline.stub && pc < end).then_some(trampoline.origin)
    })
}

pub(crate) fn decode_old_mips_kernel_call(target: u32) -> Option<(u32, u32)> {
    const OLD_FIRST_METHOD: u32 = 0xffff_fc02;
    const API_CALL_SCALE: u32 = 4;
    const API_SET_SHIFT: u32 = 8;
    const CURRENT_PROCESS_API_SET: u32 = 2;
    const PROC_TERMINATE_METHOD: u32 = 2;

    if target > OLD_FIRST_METHOD {
        return None;
    }
    let delta = OLD_FIRST_METHOD.wrapping_sub(target);
    if delta % API_CALL_SCALE != 0 {
        return None;
    }
    let encoded = delta / API_CALL_SCALE;
    let api_set = encoded >> API_SET_SHIFT;
    let method = encoded & ((1 << API_SET_SHIFT) - 1);
    (api_set == CURRENT_PROCESS_API_SET && method == PROC_TERMINATE_METHOD)
        .then_some((api_set, method))
}

// ── instruction decode helpers ────────────────────────────────────────────────

pub(crate) fn decode_addiu_zero(instruction: u32) -> Option<(u32, u32)> {
    let opcode = instruction >> 26;
    let rs = (instruction >> 21) & 0x1f;
    if opcode != 0x09 || rs != 0 {
        return None;
    }
    let rt = (instruction >> 16) & 0x1f;
    let imm = instruction as u16 as i16 as i32 as u32;
    Some((rt, imm))
}

pub(crate) fn decode_jalr_register(instruction: u32) -> Option<u32> {
    let opcode = instruction >> 26;
    let function = instruction & 0x3f;
    if opcode != 0 || function != 0x09 {
        return None;
    }
    Some((instruction >> 21) & 0x1f)
}

pub(crate) fn decode_indirect_call_register(instruction: u32) -> Option<u32> {
    let opcode = instruction >> 26;
    let function = instruction & 0x3f;
    if opcode != 0 || (function != 0x08 && function != 0x09) {
        return None;
    }
    Some((instruction >> 21) & 0x1f)
}

pub(crate) fn decode_jr_register(instruction: u32) -> Option<u32> {
    let opcode = instruction >> 26;
    let function = instruction & 0x3f;
    if opcode != 0 || function != 0x08 {
        return None;
    }
    Some((instruction >> 21) & 0x1f)
}

pub(crate) fn decode_direct_jump_target(pc: u32, instruction: u32) -> Option<u32> {
    let opcode = instruction >> 26;
    if opcode != 0x02 && opcode != 0x03 {
        return None;
    }
    let index = instruction & 0x03ff_ffff;
    Some((pc.wrapping_add(4) & 0xf000_0000) | (index << 2))
}

pub(crate) fn is_trampoline_sentinel_first_word(instruction: u32) -> bool {
    let opcode = instruction >> 26;
    let rt = (instruction >> 16) & 0x1f;
    opcode == 0x0f && rt == 26
}

// ── arch types ────────────────────────────────────────────────────────────────

/// Opaque tag type for the MIPS architecture.
pub(crate) struct MipsArch;

impl ArchHooks for MipsArch {
    type Context = MipsGuestContext;

    fn read_pc<D>(uc: &Unicorn<'_, D>) -> u32 {
        read_mips_reg(uc, RegisterMIPS::PC)
    }

    fn write_pc<D>(uc: &mut Unicorn<'_, D>, val: u32) {
        let _ = uc.reg_write(RegisterMIPS::PC, u64::from(val));
    }

    fn read_sp<D>(uc: &Unicorn<'_, D>) -> u32 {
        read_mips_reg(uc, RegisterMIPS::SP)
    }

    fn read_return_addr<D>(uc: &Unicorn<'_, D>) -> u32 {
        read_mips_reg(uc, RegisterMIPS::RA)
    }

    fn write_return_val<D>(uc: &mut Unicorn<'_, D>, val: u32) {
        let _ = uc.reg_write(RegisterMIPS::V0, u64::from(val));
    }

    fn save_context<D>(uc: &Unicorn<'_, D>) -> MipsGuestContext {
        capture_mips_gprs(uc)
    }

    fn restore_context<D>(
        uc: &mut Unicorn<'_, D>,
        ctx: &MipsGuestContext,
    ) -> crate::error::Result<()> {
        restore_mips_gprs(uc, ctx);
        Ok(())
    }

    fn is_delay_slot(pc: u32, read_u32: &dyn Fn(u32) -> Option<u32>) -> bool {
        // Check if pc-4 is a control-transfer instruction.
        let Some(prev) = pc.checked_sub(4) else {
            return false;
        };
        read_u32(prev).is_some_and(is_mips_control_transfer_instruction)
    }

    fn gpr_name(n: usize) -> &'static str {
        mips_gpr_name(n as u32)
    }
}
