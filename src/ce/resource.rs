use std::collections::{BTreeMap, BTreeSet};

use crate::ce::gwe::Rect;

const STOCK_OBJECT_BASE: u32 = 0x000b_5000;
const WHITE_BRUSH: u32 = 0;
const NULL_BRUSH: u32 = 5;
const WHITE_PEN: u32 = 6;
const NULL_PEN: u32 = 8;
const OEM_FIXED_FONT: u32 = 10;
const DEFAULT_GUI_FONT: u32 = 17;
const DC_BRUSH: u32 = 18;
const DC_PEN: u32 = 19;

pub fn stock_object_handle(index: u32) -> Option<u32> {
    let valid = matches!(
        index,
        WHITE_BRUSH..=NULL_BRUSH
            | WHITE_PEN..=NULL_PEN
            | OEM_FIXED_FONT..=DEFAULT_GUI_FONT
            | DC_BRUSH
            | DC_PEN
    );
    valid.then_some(STOCK_OBJECT_BASE | index)
}

fn stock_object_index(handle: u32) -> Option<u32> {
    (handle & 0xffff_ff00 == STOCK_OBJECT_BASE).then_some(handle & 0xff)
}

fn is_stock_font(handle: u32) -> bool {
    matches!(
        stock_object_index(handle),
        Some(OEM_FIXED_FONT..=DEFAULT_GUI_FONT)
    )
}

fn is_stock_brush(handle: u32) -> bool {
    matches!(
        stock_object_index(handle),
        Some(WHITE_BRUSH..=NULL_BRUSH | DC_BRUSH)
    )
}

fn is_stock_pen(handle: u32) -> bool {
    matches!(
        stock_object_index(handle),
        Some(WHITE_PEN..=NULL_PEN | DC_PEN)
    )
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResourceId {
    Integer(u16),
    Name(String),
    NamePtr(u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceEntry {
    pub module: u32,
    pub name: ResourceId,
    pub kind: ResourceId,
    pub data_ptr: u32,
    pub size: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceString {
    pub module: u32,
    pub id: u32,
    pub text: String,
    pub data_ptr: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitmapObject {
    pub handle: u32,
    pub width: i32,
    pub height: i32,
    pub top_down: bool,
    pub width_bytes: i32,
    pub planes: u16,
    pub bits_pixel: u16,
    pub rgb_masks: Option<[u32; 3]>,
    pub bits_ptr: u32,
    pub bits_owned: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionObject {
    pub handle: u32,
    pub rect: Rect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuObject {
    pub handle: u32,
    pub module: u32,
    pub name: ResourceId,
    pub resource_handle: Option<u32>,
    pub checked_items: BTreeMap<u32, bool>,
    pub removed_items: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceleratorEntry {
    pub flags: u8,
    pub key: u16,
    pub command: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceleratorObject {
    pub handle: u32,
    pub module: u32,
    pub name: ResourceId,
    pub resource_handle: Option<u32>,
    pub entries: Vec<AcceleratorEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontObject {
    pub handle: u32,
    pub logfont_ptr: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrushObject {
    pub handle: u32,
    pub color: u32,
    pub pattern_bitmap: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PenObject {
    pub handle: u32,
    pub style: u32,
    pub width: i32,
    pub color: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaletteObject {
    pub handle: u32,
    pub entries: Vec<[u8; 4]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DcState {
    pub selected_object: u32,
    pub selected_bitmap: u32,
    pub selected_font: u32,
    pub selected_brush: u32,
    pub selected_pen: u32,
    pub selected_palette: u32,
    pub current_pos: crate::ce::gwe::Point,
    pub bk_color: u32,
    pub bk_mode: i32,
    pub text_color: u32,
    pub text_align: u32,
}

impl Default for DcState {
    fn default() -> Self {
        Self {
            selected_object: 0,
            selected_bitmap: 0,
            selected_font: 0,
            selected_brush: 0,
            selected_pen: 0,
            selected_palette: 0,
            current_pos: crate::ce::gwe::Point { x: 0, y: 0 },
            bk_color: 0x00ff_ffff,
            bk_mode: 2,
            text_color: 0,
            text_align: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResourceSystem {
    next_handle: u32,
    next_gdi_handle: u32,
    by_key: BTreeMap<(u32, ResourceId, ResourceId), u32>,
    entries: BTreeMap<u32, ResourceEntry>,
    strings: BTreeMap<(u32, u32), ResourceString>,
    bitmaps: BTreeMap<u32, BitmapObject>,
    regions: BTreeMap<u32, RegionObject>,
    menus: BTreeMap<u32, MenuObject>,
    accelerators: BTreeMap<u32, AcceleratorObject>,
    fonts: BTreeMap<u32, FontObject>,
    brushes: BTreeMap<u32, BrushObject>,
    pens: BTreeMap<u32, PenObject>,
    palettes: BTreeMap<u32, PaletteObject>,
    memory_dcs: BTreeSet<u32>,
    dc_states: BTreeMap<u32, DcState>,
    dc_clips: BTreeMap<u32, u32>,
}

impl Default for ResourceSystem {
    fn default() -> Self {
        Self {
            next_handle: 0x0009_0000,
            next_gdi_handle: 0x000a_0000,
            by_key: BTreeMap::new(),
            entries: BTreeMap::new(),
            strings: BTreeMap::new(),
            bitmaps: BTreeMap::new(),
            regions: BTreeMap::new(),
            menus: BTreeMap::new(),
            accelerators: BTreeMap::new(),
            fonts: BTreeMap::new(),
            brushes: BTreeMap::new(),
            pens: BTreeMap::new(),
            palettes: BTreeMap::new(),
            memory_dcs: BTreeSet::new(),
            dc_states: BTreeMap::new(),
            dc_clips: BTreeMap::new(),
        }
    }
}

impl ResourceSystem {
    pub fn register(
        &mut self,
        module: u32,
        name: ResourceId,
        kind: ResourceId,
        data_ptr: u32,
        size: u32,
    ) -> u32 {
        let handle = self.next_handle;
        self.next_handle += 4;
        self.by_key
            .insert((module, name.clone(), kind.clone()), handle);
        self.entries.insert(
            handle,
            ResourceEntry {
                module,
                name,
                kind,
                data_ptr,
                size,
            },
        );
        handle
    }

    pub fn find_resource(&self, module: u32, name: ResourceId, kind: ResourceId) -> Option<u32> {
        self.by_key
            .get(&(module, name.clone(), kind.clone()))
            .copied()
            .or_else(|| {
                let (ResourceId::Integer(6), ResourceId::Integer(id)) = (&kind, &name) else {
                    return None;
                };
                let block_id = (u32::from(*id) >> 4) + 1;
                let block_id = u16::try_from(block_id).ok()?;
                self.by_key
                    .get(&(module, ResourceId::Integer(block_id), kind))
                    .copied()
            })
    }

    pub fn load_resource(&self, handle: u32) -> Option<u32> {
        Some(self.entries.get(&handle)?.data_ptr)
    }

    pub fn lock_resource(&self, handle: u32) -> Option<u32> {
        self.load_resource(handle)
    }

    pub fn sizeof_resource(&self, handle: u32) -> Option<u32> {
        Some(self.entries.get(&handle)?.size)
    }

    pub fn register_string(
        &mut self,
        module: u32,
        id: u32,
        text: impl Into<String>,
        data_ptr: Option<u32>,
    ) {
        self.strings.insert(
            (module, id),
            ResourceString {
                module,
                id,
                text: text.into(),
                data_ptr,
            },
        );
    }

    pub fn load_string(&self, module: u32, id: u32) -> Option<&ResourceString> {
        self.strings.get(&(module, id))
    }

    pub fn create_bitmap(
        &mut self,
        width: i32,
        height: i32,
        planes: u16,
        bits_pixel: u16,
        bits_ptr: u32,
    ) -> u32 {
        self.create_bitmap_with_ownership(width, height, planes, bits_pixel, bits_ptr, None, false)
    }

    pub fn create_bitmap_with_masks(
        &mut self,
        width: i32,
        height: i32,
        planes: u16,
        bits_pixel: u16,
        bits_ptr: u32,
        rgb_masks: Option<[u32; 3]>,
    ) -> u32 {
        self.create_bitmap_with_ownership(
            width, height, planes, bits_pixel, bits_ptr, rgb_masks, false,
        )
    }

    pub fn create_owned_bitmap(
        &mut self,
        width: i32,
        height: i32,
        planes: u16,
        bits_pixel: u16,
        bits_ptr: u32,
    ) -> u32 {
        self.create_bitmap_with_ownership(width, height, planes, bits_pixel, bits_ptr, None, true)
    }

    pub fn create_owned_bitmap_with_masks(
        &mut self,
        width: i32,
        height: i32,
        planes: u16,
        bits_pixel: u16,
        bits_ptr: u32,
        rgb_masks: Option<[u32; 3]>,
    ) -> u32 {
        self.create_bitmap_with_ownership(
            width, height, planes, bits_pixel, bits_ptr, rgb_masks, true,
        )
    }

    fn create_bitmap_with_ownership(
        &mut self,
        width: i32,
        height: i32,
        planes: u16,
        bits_pixel: u16,
        bits_ptr: u32,
        rgb_masks: Option<[u32; 3]>,
        bits_owned: bool,
    ) -> u32 {
        let handle = self.next_gdi_handle;
        self.next_gdi_handle += 4;
        let bits_per_row = (width.unsigned_abs() as u64) * (bits_pixel as u64);
        let width_bytes = (((bits_per_row + 31) / 32) * 4).min(i32::MAX as u64) as i32;
        self.bitmaps.insert(
            handle,
            BitmapObject {
                handle,
                width,
                height: height.abs(),
                top_down: height < 0,
                width_bytes,
                planes,
                bits_pixel,
                rgb_masks,
                bits_ptr,
                bits_owned,
            },
        );
        handle
    }

    pub fn bitmap(&self, handle: u32) -> Option<&BitmapObject> {
        self.bitmaps.get(&handle)
    }

    pub fn delete_bitmap(&mut self, handle: u32) -> bool {
        self.bitmaps.remove(&handle).is_some()
    }

    pub fn create_region(&mut self, rect: Rect) -> u32 {
        let handle = self.next_gdi_handle;
        self.next_gdi_handle += 4;
        self.regions.insert(handle, RegionObject { handle, rect });
        handle
    }

    pub fn region(&self, handle: u32) -> Option<&RegionObject> {
        self.regions.get(&handle)
    }

    pub fn set_region(&mut self, handle: u32, rect: Rect) -> bool {
        let Some(region) = self.regions.get_mut(&handle) else {
            return false;
        };
        region.rect = rect;
        true
    }

    pub fn delete_region(&mut self, handle: u32) -> bool {
        self.dc_clips.retain(|_, region| *region != handle);
        self.regions.remove(&handle).is_some()
    }

    pub fn select_clip_region(&mut self, hdc: u32, region: Option<u32>) {
        if let Some(region) = region {
            self.dc_clips.insert(hdc, region);
        } else {
            self.dc_clips.remove(&hdc);
        }
    }

    pub fn clip_region(&self, hdc: u32) -> Option<u32> {
        self.dc_clips.get(&hdc).copied()
    }

    pub fn create_menu(
        &mut self,
        module: u32,
        name: ResourceId,
        resource_handle: Option<u32>,
    ) -> u32 {
        let handle = self.next_gdi_handle;
        self.next_gdi_handle += 4;
        self.menus.insert(
            handle,
            MenuObject {
                handle,
                module,
                name,
                resource_handle,
                checked_items: BTreeMap::new(),
                removed_items: Vec::new(),
            },
        );
        handle
    }

    pub fn menu(&self, handle: u32) -> Option<&MenuObject> {
        self.menus.get(&handle)
    }

    pub fn check_menu_item(&mut self, handle: u32, item: u32, checked: bool) -> Option<u32> {
        let menu = self.menus.get_mut(&handle)?;
        let previous = menu.checked_items.insert(item, checked).unwrap_or(false);
        Some(u32::from(previous) * 0x0000_0008)
    }

    pub fn check_menu_radio_item(
        &mut self,
        handle: u32,
        first: u32,
        last: u32,
        checked: u32,
    ) -> bool {
        let Some(menu) = self.menus.get_mut(&handle) else {
            return false;
        };
        if first > last || checked < first || checked > last {
            return false;
        }
        for item in first..=last {
            menu.checked_items.insert(item, item == checked);
        }
        true
    }

    pub fn remove_menu_item(&mut self, handle: u32, item: u32) -> bool {
        let Some(menu) = self.menus.get_mut(&handle) else {
            return false;
        };
        menu.removed_items.push(item);
        menu.checked_items.remove(&item);
        true
    }

    pub fn delete_menu(&mut self, handle: u32) -> bool {
        self.menus.remove(&handle).is_some()
    }

    pub fn create_accelerator(
        &mut self,
        module: u32,
        name: ResourceId,
        resource_handle: Option<u32>,
        entries: Vec<AcceleratorEntry>,
    ) -> u32 {
        let handle = self.next_gdi_handle;
        self.next_gdi_handle += 4;
        self.accelerators.insert(
            handle,
            AcceleratorObject {
                handle,
                module,
                name,
                resource_handle,
                entries,
            },
        );
        handle
    }

    pub fn accelerator(&self, handle: u32) -> Option<&AcceleratorObject> {
        self.accelerators.get(&handle)
    }

    pub fn delete_accelerator(&mut self, handle: u32) -> bool {
        self.accelerators.remove(&handle).is_some()
    }

    pub fn create_font(&mut self, logfont_ptr: u32) -> u32 {
        let handle = self.next_gdi_handle;
        self.next_gdi_handle += 4;
        self.fonts.insert(
            handle,
            FontObject {
                handle,
                logfont_ptr,
            },
        );
        handle
    }

    pub fn create_brush(&mut self, color: u32) -> u32 {
        let handle = self.next_gdi_handle;
        self.next_gdi_handle += 4;
        self.brushes.insert(
            handle,
            BrushObject {
                handle,
                color,
                pattern_bitmap: None,
            },
        );
        handle
    }

    pub fn brush(&self, handle: u32) -> Option<&BrushObject> {
        self.brushes.get(&handle)
    }

    pub fn create_pattern_brush(&mut self, bitmap: u32) -> Option<u32> {
        if !self.bitmaps.contains_key(&bitmap) {
            return None;
        }
        let handle = self.next_gdi_handle;
        self.next_gdi_handle += 4;
        self.brushes.insert(
            handle,
            BrushObject {
                handle,
                color: 0,
                pattern_bitmap: Some(bitmap),
            },
        );
        Some(handle)
    }

    pub fn create_pen(&mut self, style: u32, width: i32, color: u32) -> u32 {
        let handle = self.next_gdi_handle;
        self.next_gdi_handle += 4;
        self.pens.insert(
            handle,
            PenObject {
                handle,
                style,
                width,
                color,
            },
        );
        handle
    }

    pub fn pen(&self, handle: u32) -> Option<&PenObject> {
        self.pens.get(&handle)
    }

    pub fn create_palette(&mut self, entries: Vec<[u8; 4]>) -> u32 {
        let handle = self.next_gdi_handle;
        self.next_gdi_handle += 4;
        self.palettes
            .insert(handle, PaletteObject { handle, entries });
        handle
    }

    pub fn palette(&self, handle: u32) -> Option<&PaletteObject> {
        self.palettes.get(&handle)
    }

    pub fn palette_mut(&mut self, handle: u32) -> Option<&mut PaletteObject> {
        self.palettes.get_mut(&handle)
    }

    pub fn create_compatible_dc(&mut self) -> u32 {
        let handle = self.next_gdi_handle;
        self.next_gdi_handle += 4;
        self.memory_dcs.insert(handle);
        self.dc_states.entry(handle).or_default();
        handle
    }

    pub fn delete_dc(&mut self, handle: u32) -> bool {
        self.dc_clips.remove(&handle);
        self.dc_states.remove(&handle);
        self.memory_dcs.remove(&handle)
    }

    pub fn select_object(&mut self, hdc: u32, object: u32) -> Option<u32> {
        if hdc == 0 || object == 0 {
            return Some(0);
        }
        let is_font = self.fonts.contains_key(&object) || is_stock_font(object);
        let is_bitmap = self.bitmaps.contains_key(&object);
        let is_brush = self.brushes.contains_key(&object) || is_stock_brush(object);
        let is_pen = self.pens.contains_key(&object) || is_stock_pen(object);
        let state = self.dc_states.entry(hdc).or_default();
        if is_font {
            let previous = state.selected_font;
            state.selected_font = object;
            state.selected_object = object;
            Some(previous)
        } else if is_bitmap {
            let previous = state.selected_bitmap;
            state.selected_bitmap = object;
            state.selected_object = object;
            Some(previous)
        } else if is_pen {
            let previous = state.selected_pen;
            state.selected_pen = object;
            state.selected_object = object;
            Some(previous)
        } else if is_brush {
            let previous = state.selected_brush;
            state.selected_brush = object;
            state.selected_object = object;
            Some(previous)
        } else {
            Some(state.selected_object)
        }
    }

    pub fn selected_bitmap(&self, hdc: u32) -> Option<u32> {
        self.dc_states
            .get(&hdc)
            .map(|state| state.selected_bitmap)
            .filter(|bitmap| *bitmap != 0)
    }

    pub fn selected_pen(&self, hdc: u32) -> Option<u32> {
        self.dc_states
            .get(&hdc)
            .map(|state| state.selected_pen)
            .filter(|pen| *pen != 0)
    }

    pub fn current_pos(&self, hdc: u32) -> Option<crate::ce::gwe::Point> {
        self.dc_states.get(&hdc).map(|state| state.current_pos)
    }

    pub fn move_to(
        &mut self,
        hdc: u32,
        point: crate::ce::gwe::Point,
    ) -> Option<crate::ce::gwe::Point> {
        if hdc == 0 {
            return None;
        }
        let state = self.dc_states.entry(hdc).or_default();
        let previous = state.current_pos;
        state.current_pos = point;
        Some(previous)
    }

    pub fn is_memory_dc(&self, hdc: u32) -> bool {
        self.memory_dcs.contains(&hdc)
    }

    pub fn select_palette(&mut self, hdc: u32, palette: u32) -> Option<u32> {
        if hdc == 0 || !self.palettes.contains_key(&palette) {
            return None;
        }
        let state = self.dc_states.entry(hdc).or_default();
        let previous = state.selected_palette;
        state.selected_palette = palette;
        Some(previous)
    }

    pub fn realize_palette(&mut self, hdc: u32) -> Option<u32> {
        if hdc == 0 {
            return None;
        }
        self.dc_states.entry(hdc).or_default();
        Some(0)
    }

    pub fn set_dc_bk_mode(&mut self, hdc: u32, mode: i32) -> Option<i32> {
        if hdc == 0 {
            return None;
        }
        let state = self.dc_states.entry(hdc).or_default();
        let previous = state.bk_mode;
        state.bk_mode = mode;
        Some(previous)
    }

    pub fn set_dc_bk_color(&mut self, hdc: u32, color: u32) -> Option<u32> {
        if hdc == 0 {
            return None;
        }
        let state = self.dc_states.entry(hdc).or_default();
        let previous = state.bk_color;
        state.bk_color = color;
        Some(previous)
    }

    pub fn set_dc_text_color(&mut self, hdc: u32, color: u32) -> Option<u32> {
        if hdc == 0 {
            return None;
        }
        let state = self.dc_states.entry(hdc).or_default();
        let previous = state.text_color;
        state.text_color = color;
        Some(previous)
    }

    pub fn set_dc_text_align(&mut self, hdc: u32, align: u32) -> Option<u32> {
        if hdc == 0 {
            return None;
        }
        let state = self.dc_states.entry(hdc).or_default();
        let previous = state.text_align;
        state.text_align = align;
        Some(previous)
    }

    pub fn delete_gdi_object(&mut self, handle: u32) -> bool {
        let mut removed = self.fonts.remove(&handle).is_some();
        removed |= self.brushes.remove(&handle).is_some();
        removed |= self.pens.remove(&handle).is_some();
        removed |= self.bitmaps.remove(&handle).is_some();
        removed |= self.delete_region(handle);
        removed |= self.palettes.remove(&handle).is_some();
        for state in self.dc_states.values_mut() {
            if state.selected_object == handle {
                state.selected_object = 0;
            }
            if state.selected_bitmap == handle {
                state.selected_bitmap = 0;
            }
            if state.selected_font == handle {
                state.selected_font = 0;
            }
            if state.selected_brush == handle {
                state.selected_brush = 0;
            }
            if state.selected_palette == handle {
                state.selected_palette = 0;
            }
        }
        removed
    }
}

impl ResourceId {
    pub fn from_guest_arg(value: u32) -> Self {
        if value <= 0xffff {
            Self::Integer(value as u16)
        } else {
            Self::NamePtr(value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ResourceId, ResourceSystem};

    #[test]
    fn rt_string_resource_lookup_falls_back_to_string_block() {
        let mut resources = ResourceSystem::default();
        let handle = resources.register(
            0x0001_0000,
            ResourceId::Integer(242),
            ResourceId::Integer(6),
            0x0040_0000,
            12,
        );

        assert_eq!(
            resources.find_resource(
                0x0001_0000,
                ResourceId::Integer(3867),
                ResourceId::Integer(6),
            ),
            Some(handle)
        );
    }
}
