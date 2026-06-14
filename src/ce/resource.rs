use std::collections::{BTreeMap, BTreeSet};

use crate::ce::gwe::{Rect, canonicalize_region_rects};

const ILC_MASK: u32 = 0x0001;
const STOCK_OBJECT_BASE: u32 = 0x000b_5000;
const MAX_EMULATED_IMAGE_LIST_IMAGES: usize = 1_048_576;
const WHITE_BRUSH: u32 = 0;
const NULL_BRUSH: u32 = 5;
const WHITE_PEN: u32 = 6;
const BLACK_PEN: u32 = 7;
const NULL_PEN: u32 = 8;
const SYSTEM_FONT: u32 = 13;
const DEFAULT_PALETTE: u32 = 15;
const DC_BRUSH: u32 = 18;
const DC_PEN: u32 = 19;
const BORDERX_PEN: u32 = 32;
const BORDERY_PEN: u32 = 33;
const DEFAULT_BITMAP_HANDLE: u32 = STOCK_OBJECT_BASE | 0x80;

pub fn stock_object_handle(index: u32) -> Option<u32> {
    let valid = matches!(
        index,
        WHITE_BRUSH..=NULL_BRUSH
            | WHITE_PEN..=NULL_PEN
            | SYSTEM_FONT
            | DEFAULT_PALETTE
            | DC_BRUSH
            | DC_PEN
            | BORDERX_PEN
            | BORDERY_PEN
    );
    valid.then_some(STOCK_OBJECT_BASE | index)
}

fn stock_object_index(handle: u32) -> Option<u32> {
    (handle & 0xffff_ff00 == STOCK_OBJECT_BASE).then_some(handle & 0xff)
}

fn is_stock_font(handle: u32) -> bool {
    matches!(stock_object_index(handle), Some(SYSTEM_FONT))
}

fn bounding_region_rect(rects: &[Rect]) -> Rect {
    rects
        .iter()
        .copied()
        .reduce(Rect::union)
        .unwrap_or_default()
}

fn merge_last_adjacent_region_rect(rects: &mut Vec<Rect>) {
    let len = rects.len();
    if len < 2 {
        return;
    }
    let previous = rects[len - 2];
    let last = rects[len - 1];
    if previous.left == last.left && previous.right == last.right && previous.bottom == last.top {
        rects[len - 2].bottom = last.bottom;
        rects.pop();
    }
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
        Some(WHITE_PEN..=NULL_PEN | DC_PEN | BORDERX_PEN | BORDERY_PEN)
    )
}

fn is_stock_palette(handle: u32) -> bool {
    matches!(stock_object_index(handle), Some(DEFAULT_PALETTE))
}

fn is_default_bitmap(handle: u32) -> bool {
    handle == DEFAULT_BITMAP_HANDLE
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
    pub color_table: Vec<[u8; 4]>,
    pub bits_ptr: u32,
    pub bits_owned: bool,
    pub bits_writable: bool,
    pub dib_section: bool,
    pub section_handle: u32,
    pub section_offset: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IconObject {
    pub handle: u32,
    pub is_icon: bool,
    pub x_hotspot: u32,
    pub y_hotspot: u32,
    pub mask_bitmap: u32,
    pub color_bitmap: u32,
    pub owns_mask_bitmap: bool,
    pub owns_color_bitmap: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionObject {
    pub handle: u32,
    pub rect: Rect,
    pub rects: Vec<Rect>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuObject {
    pub handle: u32,
    pub module: u32,
    pub name: ResourceId,
    pub resource_handle: Option<u32>,
    pub popup: bool,
    pub items: Vec<MenuItem>,
    pub checked_items: BTreeMap<u32, bool>,
    pub removed_items: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PopupMenuTracking {
    pub menu: u32,
    pub flags: u32,
    pub x: i32,
    pub y: i32,
    pub hwnd: u32,
    pub exclude_rect: Option<[i32; 4]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PopupMenuNotification {
    pub hwnd: u32,
    pub msg: u32,
    pub wparam: u32,
    pub lparam: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PopupMenuCommandSelection {
    pub command: u32,
    pub submenus: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuItem {
    pub id: u32,
    pub item_type: u32,
    pub state: u32,
    pub submenu: u32,
    pub checked_bitmap: u32,
    pub unchecked_bitmap: u32,
    pub data: u32,
    pub text: Option<String>,
}

impl MenuItem {
    pub fn from_insert_flags(flags: u32, id_or_submenu: u32, text: Option<String>) -> Self {
        let popup = flags & MF_POPUP != 0;
        Self {
            id: if popup { u32::MAX } else { id_or_submenu },
            item_type: flags & MENU_ITEM_TYPE_MASK,
            state: flags & MENU_ITEM_STATE_MASK,
            submenu: if popup { id_or_submenu } else { 0 },
            checked_bitmap: 0,
            unchecked_bitmap: 0,
            data: 0,
            text,
        }
    }
}

pub const MF_BYPOSITION: u32 = 0x0000_0400;
pub const MF_POPUP: u32 = 0x0000_0010;
pub const MF_SEPARATOR: u32 = 0x0000_0800;
pub const MF_ENABLED: u32 = 0x0000_0000;
pub const MF_DISABLED: u32 = 0x0000_0002;
pub const MF_CHECKED: u32 = 0x0000_0008;
pub const MF_GRAYED: u32 = 0x0000_0001;
pub const MF_OWNERDRAW: u32 = 0x0000_0100;
pub const MF_MENUBARBREAK: u32 = 0x0000_0020;
pub const MF_MENUBREAK: u32 = 0x0000_0040;
pub const MF_HILITE: u32 = 0x0000_0080;
pub const MFT_RADIOCHECK: u32 = 0x0000_0200;
pub const MFS_DEFAULT: u32 = 0x0000_1000;
const MENU_ITEM_TYPE_MASK: u32 =
    MF_SEPARATOR | MF_OWNERDRAW | MF_MENUBARBREAK | MF_MENUBREAK | MFT_RADIOCHECK;
const MENU_ITEM_STATE_MASK: u32 = MF_CHECKED | MF_DISABLED | MF_GRAYED | MF_HILITE | MFS_DEFAULT;

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
pub struct ImageListObject {
    pub handle: u32,
    pub width: i32,
    pub height: i32,
    pub flags: u32,
    pub grow: i32,
    pub bk_color: u32,
    pub images: Vec<ImageListImage>,
    pub overlays: BTreeMap<u32, ImageListOverlay>,
    pub last_draw: Option<ImageListDraw>,
    pub last_dither_copy: Option<ImageListDitherCopy>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageListOverlay {
    pub image_index: i32,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub flags: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImageListImage {
    pub bitmap: u32,
    pub mask: u32,
    pub icon: u32,
    pub transparent_color: Option<u32>,
    pub source_x: i32,
    pub source_y: i32,
}

fn overlay_icon_handle(base_icon: u32, overlay_index: i32) -> u32 {
    let overlay = (overlay_index.saturating_add(1) as u32).min(0xff);
    base_icon | (overlay << 24)
}

fn image_list_bitmap_strip_count(bitmap_width: i32, image_width: i32) -> i32 {
    if bitmap_width <= 0 || image_width <= 0 || bitmap_width < image_width {
        return 0;
    }
    bitmap_width / image_width
}

fn remove_image_list_image_at(list: &mut ImageListObject, index: usize) {
    list.images.remove(index);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageListImageInfo {
    pub bitmap: u32,
    pub mask: u32,
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageListDraw {
    pub image_list: u32,
    pub index: i32,
    pub hdc: u32,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub flags: u32,
    pub overlay_image: Option<i32>,
    /// Foreground (blend) color as COLORREF (CLR_DEFAULT = 0xFF000000, CLR_NONE = 0xFFFFFFFF).
    pub rgb_fg: u32,
    /// Background color (COLORREF). CLR_NONE = transparent; CLR_DEFAULT = image list bk.
    /// When ILD_TRANSPARENT is not set and this is not CLR_NONE, transparent pixels are
    /// filled with this color instead of being skipped (matching CE DrawIndirect ROP_PatMask).
    pub rgb_bk: u32,
    /// Additional source bitmap x offset from IMAGELISTDRAWPARAMS.xBitmap.
    pub x_bitmap: i32,
    /// Additional source bitmap y offset from IMAGELISTDRAWPARAMS.yBitmap.
    pub y_bitmap: i32,
    /// Raster operation from IMAGELISTDRAWPARAMS.dwRop, used when ILD_ROP is set.
    pub rop: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageListDitherCopy {
    pub dst_image_list: u32,
    pub dst_index: i32,
    pub x: i32,
    pub y: i32,
    pub src_image_list: u32,
    pub src_index: i32,
    pub flags: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageListDragState {
    pub image_list: u32,
    pub index: i32,
    pub hotspot_x: i32,
    pub hotspot_y: i32,
    pub lock_hwnd: u32,
    pub x: i32,
    pub y: i32,
    pub visible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontObject {
    pub handle: u32,
    pub logfont_ptr: u32,
    pub height: i32,
    pub width: i32,
    pub escapement: i32,
    pub orientation: i32,
    pub weight: i32,
    pub italic: bool,
    pub underline: bool,
    pub strikeout: bool,
    pub charset: u8,
    pub out_precision: u8,
    pub clip_precision: u8,
    pub quality: u8,
    pub pitch_and_family: u8,
    pub face_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrushObject {
    pub handle: u32,
    pub color: u32,
    pub pattern_bitmap: Option<u32>,
    pub owns_pattern_bitmap: bool,
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
    pub brush_origin: crate::ce::gwe::Point,
    pub bk_color: u32,
    pub bk_mode: i32,
    pub text_color: u32,
    pub text_align: u32,
    pub rop2: i32,
    pub stretch_blt_mode: i32,
    pub text_char_extra: i32,
    pub layout: u32,
    pub poly_fill_mode: i32,
}

pub const POLY_FILL_ALTERNATE: i32 = 1;
pub const POLY_FILL_WINDING: i32 = 2;

impl Default for DcState {
    fn default() -> Self {
        Self {
            selected_object: 0,
            selected_bitmap: DEFAULT_BITMAP_HANDLE,
            selected_font: stock_object_handle(SYSTEM_FONT).unwrap_or(0),
            selected_brush: stock_object_handle(WHITE_BRUSH).unwrap_or(0),
            selected_pen: stock_object_handle(BLACK_PEN).unwrap_or(0),
            selected_palette: stock_object_handle(DEFAULT_PALETTE).unwrap_or(0),
            current_pos: crate::ce::gwe::Point { x: 0, y: 0 },
            brush_origin: crate::ce::gwe::Point { x: 0, y: 0 },
            bk_color: 0x00ff_ffff,
            bk_mode: 2,
            text_color: 0,
            text_align: 0,
            rop2: 13,
            stretch_blt_mode: 1, // BLACKONWHITE
            text_char_extra: 0,
            layout: 0, // left-to-right
            poly_fill_mode: POLY_FILL_ALTERNATE,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResourceSystem {
    next_handle: u32,
    next_gdi_handle: u32,
    next_icon_handle: u32,
    by_key: BTreeMap<(u32, ResourceId, ResourceId), u32>,
    entries: BTreeMap<u32, ResourceEntry>,
    strings: BTreeMap<(u32, u32), ResourceString>,
    bitmaps: BTreeMap<u32, BitmapObject>,
    icons: BTreeMap<u32, IconObject>,
    regions: BTreeMap<u32, RegionObject>,
    menus: BTreeMap<u32, MenuObject>,
    accelerators: BTreeMap<u32, AcceleratorObject>,
    image_lists: BTreeMap<u32, ImageListObject>,
    fonts: BTreeMap<u32, FontObject>,
    brushes: BTreeMap<u32, BrushObject>,
    pens: BTreeMap<u32, PenObject>,
    palettes: BTreeMap<u32, PaletteObject>,
    memory_dcs: BTreeSet<u32>,
    dc_states: BTreeMap<u32, DcState>,
    dc_save_stacks: BTreeMap<u32, Vec<DcState>>,
    dc_clips: BTreeMap<u32, u32>,
    last_popup_tracking: Option<PopupMenuTracking>,
    popup_notifications: Vec<PopupMenuNotification>,
    image_list_drag: Option<ImageListDragState>,
    image_list_drag_x: i32,
    image_list_drag_y: i32,
    image_list_drag_hotspot_x: i32,
    image_list_drag_hotspot_y: i32,
    image_list_drag_lock_hwnd: u32,
}

impl Default for ResourceSystem {
    fn default() -> Self {
        Self {
            next_handle: 0x0009_0000,
            next_gdi_handle: 0x000a_0000,
            next_icon_handle: 0x000c_8000,
            by_key: BTreeMap::new(),
            entries: BTreeMap::new(),
            strings: BTreeMap::new(),
            bitmaps: BTreeMap::new(),
            icons: BTreeMap::new(),
            regions: BTreeMap::new(),
            menus: BTreeMap::new(),
            accelerators: BTreeMap::new(),
            image_lists: BTreeMap::new(),
            fonts: BTreeMap::new(),
            brushes: BTreeMap::new(),
            pens: BTreeMap::new(),
            palettes: BTreeMap::new(),
            memory_dcs: BTreeSet::new(),
            dc_states: BTreeMap::new(),
            dc_save_stacks: BTreeMap::new(),
            dc_clips: BTreeMap::new(),
            last_popup_tracking: None,
            popup_notifications: Vec::new(),
            image_list_drag: None,
            image_list_drag_x: 0,
            image_list_drag_y: 0,
            image_list_drag_hotspot_x: 0,
            image_list_drag_hotspot_y: 0,
            image_list_drag_lock_hwnd: 0,
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

    pub fn resource_entry(&self, handle: u32) -> Option<&ResourceEntry> {
        self.entries.get(&handle)
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

    pub fn remove_module_resources(&mut self, module: u32) -> usize {
        let resource_handles = self
            .entries
            .iter()
            .filter_map(|(handle, entry)| (entry.module == module).then_some(*handle))
            .collect::<Vec<_>>();
        let resource_count = resource_handles.len();
        for handle in resource_handles {
            self.entries.remove(&handle);
        }
        self.by_key
            .retain(|(entry_module, _name, _kind), _handle| *entry_module != module);

        let string_keys = self
            .strings
            .keys()
            .filter(|(entry_module, _id)| *entry_module == module)
            .copied()
            .collect::<Vec<_>>();
        let string_count = string_keys.len();
        for key in string_keys {
            self.strings.remove(&key);
        }

        resource_count + string_count
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
        self.create_bitmap_with_flags(
            width, height, planes, bits_pixel, bits_ptr, None, false, true,
        )
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
        self.create_bitmap_with_flags(
            width, height, planes, bits_pixel, bits_ptr, rgb_masks, false, true,
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
        self.create_bitmap_with_flags(
            width, height, planes, bits_pixel, bits_ptr, None, true, true,
        )
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
        self.create_bitmap_with_flags(
            width, height, planes, bits_pixel, bits_ptr, rgb_masks, true, true,
        )
    }

    pub fn create_readonly_owned_bitmap(
        &mut self,
        width: i32,
        height: i32,
        planes: u16,
        bits_pixel: u16,
        bits_ptr: u32,
    ) -> u32 {
        self.create_bitmap_with_flags(
            width, height, planes, bits_pixel, bits_ptr, None, true, false,
        )
    }

    pub fn create_readonly_owned_bitmap_with_masks(
        &mut self,
        width: i32,
        height: i32,
        planes: u16,
        bits_pixel: u16,
        bits_ptr: u32,
        rgb_masks: Option<[u32; 3]>,
    ) -> u32 {
        self.create_bitmap_with_flags(
            width, height, planes, bits_pixel, bits_ptr, rgb_masks, true, false,
        )
    }

    fn create_bitmap_with_flags(
        &mut self,
        width: i32,
        height: i32,
        planes: u16,
        bits_pixel: u16,
        bits_ptr: u32,
        rgb_masks: Option<[u32; 3]>,
        bits_owned: bool,
        bits_writable: bool,
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
                color_table: Vec::new(),
                bits_ptr,
                bits_owned,
                bits_writable,
                dib_section: false,
                section_handle: 0,
                section_offset: 0,
            },
        );
        handle
    }

    pub fn bitmap(&self, handle: u32) -> Option<&BitmapObject> {
        self.bitmaps.get(&handle)
    }

    pub fn bitmap_mut(&mut self, handle: u32) -> Option<&mut BitmapObject> {
        self.bitmaps.get_mut(&handle)
    }

    pub fn delete_bitmap(&mut self, handle: u32) -> bool {
        self.bitmaps.remove(&handle).is_some()
    }

    pub fn create_icon(
        &mut self,
        is_icon: bool,
        x_hotspot: u32,
        y_hotspot: u32,
        mask_bitmap: u32,
        color_bitmap: u32,
    ) -> Option<u32> {
        self.create_icon_with_bitmap_ownership(
            is_icon,
            x_hotspot,
            y_hotspot,
            mask_bitmap,
            color_bitmap,
            false,
            false,
        )
    }

    pub fn create_icon_with_bitmap_ownership(
        &mut self,
        is_icon: bool,
        x_hotspot: u32,
        y_hotspot: u32,
        mask_bitmap: u32,
        color_bitmap: u32,
        owns_mask_bitmap: bool,
        owns_color_bitmap: bool,
    ) -> Option<u32> {
        if (mask_bitmap != 0 && !self.bitmaps.contains_key(&mask_bitmap))
            || (color_bitmap != 0 && !self.bitmaps.contains_key(&color_bitmap))
            || (mask_bitmap == 0 && color_bitmap == 0)
        {
            return None;
        }
        let handle = self.next_icon_handle;
        self.next_icon_handle += 4;
        self.icons.insert(
            handle,
            IconObject {
                handle,
                is_icon,
                x_hotspot,
                y_hotspot,
                mask_bitmap,
                color_bitmap,
                owns_mask_bitmap: owns_mask_bitmap && mask_bitmap != 0,
                owns_color_bitmap: owns_color_bitmap && color_bitmap != 0,
            },
        );
        Some(handle)
    }

    pub fn icon(&self, handle: u32) -> Option<&IconObject> {
        self.icons.get(&handle)
    }

    pub fn delete_icon(&mut self, handle: u32) -> bool {
        self.icons.remove(&handle).is_some()
    }

    pub fn create_region(&mut self, rect: Rect) -> u32 {
        let handle = self.next_gdi_handle;
        self.next_gdi_handle += 4;
        let rect = rect.normalized();
        let rects = if rect.is_empty() {
            Vec::new()
        } else {
            vec![rect]
        };
        self.regions.insert(
            handle,
            RegionObject {
                handle,
                rect,
                rects,
            },
        );
        handle
    }

    pub fn region(&self, handle: u32) -> Option<&RegionObject> {
        self.regions.get(&handle)
    }

    pub fn set_region(&mut self, handle: u32, rect: Rect) -> bool {
        let rects = if rect.is_empty() {
            Vec::new()
        } else {
            vec![rect]
        };
        self.set_region_rects(handle, rects)
    }

    pub fn set_region_rects(&mut self, handle: u32, rects: Vec<Rect>) -> bool {
        let Some(region) = self.regions.get_mut(&handle) else {
            return false;
        };
        let rects = canonicalize_region_rects(rects);
        region.rect = bounding_region_rect(&rects);
        region.rects = rects;
        true
    }

    pub fn union_region_with_rect(&mut self, handle: u32, rect: Rect) -> bool {
        let Some(region) = self.regions.get_mut(&handle) else {
            return false;
        };
        let rect = rect.normalized();
        if rect.is_empty() {
            return true;
        }
        if region.rects.iter().any(|current| {
            rect.left >= current.left
                && rect.top >= current.top
                && rect.right <= current.right
                && rect.bottom <= current.bottom
        }) {
            return true;
        }
        if let Some(last) = region.rects.last_mut() {
            if last.top == rect.top
                && last.bottom == rect.bottom
                && rect.left >= last.left
                && rect.left <= last.right
            {
                last.right = last.right.max(rect.right);
                merge_last_adjacent_region_rect(&mut region.rects);
                region.rect = region.rect.union(rect);
                return true;
            }
            if last.left == rect.left && last.right == rect.right && last.bottom == rect.top {
                last.bottom = rect.bottom;
                region.rect = region.rect.union(rect);
                return true;
            }
            if rect.top > last.top || (rect.top == last.top && rect.left >= last.left) {
                region.rects.push(rect);
                merge_last_adjacent_region_rect(&mut region.rects);
                region.rect = region.rect.union(rect);
                return true;
            }
        } else {
            region.rect = rect;
            region.rects.push(rect);
            return true;
        }

        let mut rects = region.rects.clone();
        rects.push(rect);
        let rects = canonicalize_region_rects(rects);
        region.rect = bounding_region_rect(&rects);
        region.rects = rects;
        true
    }

    pub fn offset_region(&mut self, handle: u32, dx: i32, dy: i32) -> Option<bool> {
        let region = self.regions.get_mut(&handle)?;
        let shifted: Vec<Rect> = region.rects.iter().map(|r| r.offset(dx, dy)).collect();
        let bounding = if shifted.is_empty() {
            Rect::default()
        } else {
            crate::ce::gwe::bounding_region_rect_pub(&shifted)
        };
        region.rects = shifted;
        region.rect = bounding;
        Some(!region.rects.is_empty())
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
                popup: false,
                items: Vec::new(),
                checked_items: BTreeMap::new(),
                removed_items: Vec::new(),
            },
        );
        handle
    }

    pub fn create_popup_menu(&mut self) -> u32 {
        let handle = self.create_menu(0, ResourceId::Integer(0), None);
        if let Some(menu) = self.menus.get_mut(&handle) {
            menu.popup = true;
        }
        handle
    }

    pub fn menu(&self, handle: u32) -> Option<&MenuObject> {
        self.menus.get(&handle)
    }

    pub fn append_menu_item(&mut self, handle: u32, item: MenuItem) -> bool {
        let Some(menu) = self.menus.get_mut(&handle) else {
            return false;
        };
        menu.items.push(item);
        true
    }

    pub fn insert_menu_item(
        &mut self,
        handle: u32,
        position_or_id: u32,
        flags: u32,
        item: MenuItem,
    ) -> bool {
        let Some(menu) = self.menus.get_mut(&handle) else {
            return false;
        };
        let index = if flags & MF_BYPOSITION != 0 {
            position_or_id as usize
        } else {
            match menu.items.iter().position(|item| item.id == position_or_id) {
                Some(index) => index,
                None => return false,
            }
        };
        if index > menu.items.len() {
            return false;
        }
        menu.items.insert(index, item);
        true
    }

    pub fn set_menu_item(
        &mut self,
        handle: u32,
        item_or_pos: u32,
        by_position: bool,
        item: MenuItem,
    ) -> bool {
        let Some(menu) = self.menus.get_mut(&handle) else {
            return false;
        };
        let Some(index) = menu_item_index(menu, item_or_pos, by_position) else {
            return false;
        };
        menu.items[index] = item;
        true
    }

    pub fn get_menu_item(
        &self,
        handle: u32,
        item_or_pos: u32,
        by_position: bool,
    ) -> Option<&MenuItem> {
        let menu = self.menus.get(&handle)?;
        let index = menu_item_index(menu, item_or_pos, by_position)?;
        menu.items.get(index)
    }

    pub fn get_sub_menu(&self, handle: u32, position: u32) -> Option<u32> {
        let menu = self.menus.get(&handle)?;
        let item = menu.items.get(position as usize)?;
        (item.submenu != 0).then_some(item.submenu)
    }

    pub fn enable_menu_item(&mut self, handle: u32, item_or_pos: u32, flags: u32) -> Option<u32> {
        let menu = self.menus.get_mut(&handle)?;
        let by_position = flags & MF_BYPOSITION != 0;
        let index = menu_item_index(menu, item_or_pos, by_position)?;
        let item = menu.items.get_mut(index)?;
        let previous = item.state & (MF_DISABLED | MF_GRAYED);
        item.state =
            (item.state & !(MF_DISABLED | MF_GRAYED)) | (flags & (MF_DISABLED | MF_GRAYED));
        Some(previous)
    }

    pub fn check_menu_item(&mut self, handle: u32, item: u32, flags: u32) -> Option<u32> {
        let menu = self.menus.get_mut(&handle)?;
        let by_position = flags & MF_BYPOSITION != 0;
        let index = menu_item_index(menu, item, by_position)?;
        let menu_item = menu.items.get_mut(index)?;
        let previous = menu_item.state & MF_CHECKED;
        let checked = flags & MF_CHECKED != 0;
        if checked {
            menu_item.state |= MF_CHECKED;
        } else {
            menu_item.state &= !MF_CHECKED;
        }
        menu.checked_items.insert(menu_item.id, checked);
        Some(previous)
    }

    pub fn set_menu_hilite_index(&mut self, handle: u32, index: Option<usize>) -> bool {
        let Some(menu) = self.menus.get_mut(&handle) else {
            return false;
        };
        for (item_index, item) in menu.items.iter_mut().enumerate() {
            let enabled =
                item.item_type & MF_SEPARATOR == 0 && item.state & (MF_DISABLED | MF_GRAYED) == 0;
            if enabled && Some(item_index) == index {
                item.state |= MF_HILITE;
            } else {
                item.state &= !MF_HILITE;
            }
        }
        true
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
            if let Some(menu_item) = menu.items.iter_mut().find(|menu_item| menu_item.id == item) {
                if item == checked {
                    menu_item.state |= MF_CHECKED;
                } else {
                    menu_item.state &= !MF_CHECKED;
                }
            }
        }
        true
    }

    pub fn remove_menu_item(&mut self, handle: u32, item: u32, flags: u32) -> bool {
        let Some(menu) = self.menus.get_mut(&handle) else {
            return false;
        };
        let by_position = flags & MF_BYPOSITION != 0;
        let Some(index) = menu_item_index(menu, item, by_position) else {
            return false;
        };
        let removed = menu.items.remove(index);
        menu.removed_items.push(item);
        menu.checked_items.remove(&item);
        if removed.id != item {
            menu.checked_items.remove(&removed.id);
        }
        true
    }

    pub fn delete_menu(&mut self, handle: u32) -> bool {
        let removed = self.menus.remove(&handle).is_some();
        if removed
            && self
                .last_popup_tracking
                .as_ref()
                .is_some_and(|tracking| tracking.menu == handle)
        {
            self.last_popup_tracking = None;
        }
        removed
    }

    pub fn track_popup_menu(&mut self, tracking: PopupMenuTracking) -> bool {
        if !self.menus.contains_key(&tracking.menu) {
            return false;
        }
        self.last_popup_tracking = Some(tracking);
        true
    }

    pub fn popup_menu_return_command(&self, handle: u32) -> Option<u32> {
        self.popup_menu_command_selection(handle)
            .map(|selection| selection.command)
    }

    pub fn popup_menu_command_selection(&self, handle: u32) -> Option<PopupMenuCommandSelection> {
        let mut visited = BTreeSet::new();
        self.popup_menu_default_command(handle, &mut visited)
            .or_else(|| {
                let mut visited = BTreeSet::new();
                self.popup_menu_first_command(handle, &mut visited)
            })
    }

    fn popup_menu_default_command(
        &self,
        handle: u32,
        visited: &mut BTreeSet<u32>,
    ) -> Option<PopupMenuCommandSelection> {
        if !visited.insert(handle) {
            return None;
        }
        let menu = self.menus.get(&handle)?;
        for item in menu
            .items
            .iter()
            .filter(|item| popup_menu_item_is_enabled(item) && item.state & MFS_DEFAULT != 0)
        {
            if popup_menu_item_is_selectable(item) {
                return Some(PopupMenuCommandSelection {
                    command: item.id,
                    submenus: Vec::new(),
                });
            }
            if item.submenu != 0 {
                let mut default_visited = visited.clone();
                if let Some(mut selection) =
                    self.popup_menu_default_command(item.submenu, &mut default_visited)
                {
                    selection.submenus.insert(0, item.submenu);
                    return Some(selection);
                }
                let mut first_visited = visited.clone();
                if let Some(mut selection) =
                    self.popup_menu_first_command(item.submenu, &mut first_visited)
                {
                    selection.submenus.insert(0, item.submenu);
                    return Some(selection);
                }
            }
        }
        for item in menu
            .items
            .iter()
            .filter(|item| popup_menu_item_is_enabled(item) && item.submenu != 0)
        {
            let mut nested_visited = visited.clone();
            if let Some(mut selection) =
                self.popup_menu_default_command(item.submenu, &mut nested_visited)
            {
                selection.submenus.insert(0, item.submenu);
                return Some(selection);
            }
        }
        None
    }

    fn popup_menu_first_command(
        &self,
        handle: u32,
        visited: &mut BTreeSet<u32>,
    ) -> Option<PopupMenuCommandSelection> {
        if !visited.insert(handle) {
            return None;
        }
        let menu = self.menus.get(&handle)?;
        for item in menu
            .items
            .iter()
            .filter(|item| popup_menu_item_is_enabled(item))
        {
            if popup_menu_item_is_selectable(item) {
                return Some(PopupMenuCommandSelection {
                    command: item.id,
                    submenus: Vec::new(),
                });
            }
            if item.submenu != 0 {
                let mut nested_visited = visited.clone();
                if let Some(mut selection) =
                    self.popup_menu_first_command(item.submenu, &mut nested_visited)
                {
                    selection.submenus.insert(0, item.submenu);
                    return Some(selection);
                }
            }
        }
        None
    }

    pub fn last_popup_tracking(&self) -> Option<&PopupMenuTracking> {
        self.last_popup_tracking.as_ref()
    }

    pub fn record_popup_notification(&mut self, notification: PopupMenuNotification) {
        self.popup_notifications.push(notification);
    }

    pub fn popup_notifications(&self) -> &[PopupMenuNotification] {
        &self.popup_notifications
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

    pub fn create_image_list(
        &mut self,
        width: i32,
        height: i32,
        flags: u32,
        initial: i32,
        grow: i32,
    ) -> Option<u32> {
        const ILC_COLORMASK: u32 = 0x00fe;
        const ILC_SHARED: u32 = 0x0100;
        const ILC_PALETTE: u32 = 0x0800;
        const ILC_MIRROR: u32 = 0x2000;
        const ILC_VIRTUAL: u32 = 0x8000;
        const ILC_VALID: u32 =
            ILC_MASK | ILC_COLORMASK | ILC_SHARED | ILC_PALETTE | ILC_MIRROR | ILC_VIRTUAL;
        if width <= 0 || height <= 0 {
            return None;
        }
        if flags & !ILC_VALID != 0 {
            return None;
        }
        let flags = if flags & ILC_COLORMASK == 0 {
            flags | ILC_COLORMASK
        } else {
            flags
        };
        let handle = self.next_gdi_handle;
        self.next_gdi_handle += 4;
        self.image_lists.insert(
            handle,
            ImageListObject {
                handle,
                width,
                height,
                flags,
                grow,
                bk_color: 0xffff_ffff,
                images: Vec::with_capacity(initial.max(0) as usize),
                overlays: BTreeMap::new(),
                last_draw: None,
                last_dither_copy: None,
            },
        );
        Some(handle)
    }

    pub fn create_shell_system_image_list(&mut self, handle: u32, width: i32, height: i32) {
        self.image_lists
            .entry(handle)
            .or_insert_with(|| ImageListObject {
                handle,
                width,
                height,
                flags: 0,
                grow: 0,
                bk_color: 0xffff_ffff,
                images: Vec::new(),
                overlays: BTreeMap::new(),
                last_draw: None,
                last_dither_copy: None,
            });
    }

    pub fn image_list(&self, handle: u32) -> Option<&ImageListObject> {
        self.image_lists.get(&handle)
    }

    pub fn image_list_mut(&mut self, handle: u32) -> Option<&mut ImageListObject> {
        self.image_lists.get_mut(&handle)
    }

    pub fn set_image_list_size(&mut self, handle: u32, width: i32, height: i32) -> Option<bool> {
        let list = self.image_lists.get_mut(&handle)?;
        if list.width == width && list.height == height {
            return Some(false);
        }
        list.width = width;
        list.height = height;
        list.images.clear();
        list.overlays.clear();
        Some(true)
    }

    pub fn destroy_image_list(&mut self, handle: u32) -> bool {
        let removed = self.image_lists.remove(&handle).is_some();
        if removed
            && self
                .image_list_drag
                .is_some_and(|drag| drag.image_list == handle)
        {
            self.image_list_drag = None;
        }
        removed
    }

    pub fn duplicate_image_list(&mut self, handle: u32) -> Option<u32> {
        let images = self.image_lists.get(&handle)?.images.clone();
        self.duplicate_image_list_with_images(handle, images)
    }

    pub fn duplicate_image_list_with_images(
        &mut self,
        handle: u32,
        images: Vec<ImageListImage>,
    ) -> Option<u32> {
        let mut duplicate = self.image_lists.get(&handle)?.clone();
        let new_handle = self.next_gdi_handle;
        self.next_gdi_handle += 4;
        duplicate.handle = new_handle;
        duplicate.images = images;
        duplicate.last_draw = None;
        duplicate.last_dither_copy = None;
        self.image_lists.insert(new_handle, duplicate);
        Some(new_handle)
    }

    pub fn merge_image_list_images(
        &mut self,
        first_handle: u32,
        first_index: i32,
        second_handle: u32,
        second_index: i32,
        dx: i32,
        dy: i32,
    ) -> Option<u32> {
        if first_index < 0 || second_index < 0 {
            return None;
        }
        let first = self.image_lists.get(&first_handle)?;
        let first_image = first.images.get(first_index as usize)?.clone();
        let first_width = first.width;
        let first_height = first.height;
        let first_flags = first.flags;
        let first_grow = first.grow;
        let first_bk_color = first.bk_color;
        let overlays = first.overlays.clone();
        let second = self.image_lists.get(&second_handle)?;
        let second_image = second.images.get(second_index as usize)?.clone();
        let second_width = second.width;
        let second_height = second.height;
        let second_flags = second.flags;

        let left = 0_i64.min(i64::from(dx));
        let top = 0_i64.min(i64::from(dy));
        let right = i64::from(first_width).max(i64::from(dx) + i64::from(second_width));
        let bottom = i64::from(first_height).max(i64::from(dy) + i64::from(second_height));
        let width = i32::try_from((right - left).max(1)).unwrap_or(i32::MAX);
        let height = i32::try_from((bottom - top).max(1)).unwrap_or(i32::MAX);

        const ILC_MASK: u32 = 0x0001;
        const ILC_COLORMASK: u32 = 0x00fe;
        const ILC_COLOR16: u32 = 0x0010;
        const ILC_COLOR32: u32 = 0x0020;
        const ILC_COLORDDB: u32 = 0x00fe;
        let first_color = first_flags & ILC_COLORMASK;
        let mut second_color = second_flags & ILC_COLORMASK;
        if (first_color == ILC_COLOR16 || first_color == ILC_COLOR32)
            && second_color == ILC_COLORDDB
        {
            second_color = first_color;
        }
        let flags = ILC_MASK
            | ((first_flags | second_flags) & !ILC_COLORMASK)
            | first_color.max(second_color);

        let handle = self.next_gdi_handle;
        self.next_gdi_handle += 4;
        self.image_lists.insert(
            handle,
            ImageListObject {
                handle,
                width,
                height,
                flags,
                grow: first_grow,
                bk_color: first_bk_color,
                images: vec![first_image, second_image],
                overlays,
                last_draw: None,
                last_dither_copy: None,
            },
        );
        Some(handle)
    }

    pub fn add_image_list_image(&mut self, handle: u32, bitmap: u32, mask: u32) -> Option<i32> {
        if bitmap == 0 {
            return None;
        }
        let bitmap_width = self
            .bitmaps
            .get(&bitmap)
            .map(|bitmap| bitmap.width.abs())
            .unwrap_or(0);
        let list = self.image_lists.get_mut(&handle)?;
        let index = list.images.len();
        let count = image_list_bitmap_strip_count(bitmap_width, list.width);
        if count == 0 {
            return None;
        }
        for strip in 0..count {
            list.images.push(ImageListImage {
                bitmap,
                mask,
                icon: 0,
                transparent_color: None,
                source_x: strip.saturating_mul(list.width),
                source_y: 0,
            });
        }
        i32::try_from(index).ok()
    }

    pub fn add_masked_image_list_image(
        &mut self,
        handle: u32,
        bitmap: u32,
        transparent_color: u32,
    ) -> Option<i32> {
        if bitmap == 0 {
            return None;
        }
        let bitmap_width = self
            .bitmaps
            .get(&bitmap)
            .map(|bitmap| bitmap.width.abs())
            .unwrap_or(0);
        let list = self.image_lists.get_mut(&handle)?;
        let index = list.images.len();
        let count = image_list_bitmap_strip_count(bitmap_width, list.width);
        if count == 0 {
            return None;
        }
        for strip in 0..count {
            list.images.push(ImageListImage {
                bitmap,
                mask: transparent_color,
                icon: 0,
                transparent_color: (transparent_color != 0xffff_ffff).then_some(transparent_color),
                source_x: strip.saturating_mul(list.width),
                source_y: 0,
            });
        }
        i32::try_from(index).ok()
    }

    pub fn add_masked_image_list_image_with_mask(
        &mut self,
        handle: u32,
        bitmap: u32,
        mask: u32,
        transparent_color: u32,
    ) -> Option<i32> {
        if bitmap == 0 {
            return None;
        }
        let bitmap_width = self
            .bitmaps
            .get(&bitmap)
            .map(|bitmap| bitmap.width.abs())
            .unwrap_or(0);
        let list = self.image_lists.get_mut(&handle)?;
        let index = list.images.len();
        let count = image_list_bitmap_strip_count(bitmap_width, list.width);
        if count == 0 {
            return None;
        }
        for strip in 0..count {
            list.images.push(ImageListImage {
                bitmap,
                mask,
                icon: 0,
                transparent_color: (transparent_color != 0xffff_ffff).then_some(transparent_color),
                source_x: strip.saturating_mul(list.width),
                source_y: 0,
            });
        }
        i32::try_from(index).ok()
    }

    pub fn replace_image_list_image(
        &mut self,
        handle: u32,
        index: i32,
        bitmap: u32,
        mask: u32,
    ) -> Option<bool> {
        if index < 0 || bitmap == 0 {
            return Some(false);
        }
        let list = self.image_lists.get_mut(&handle)?;
        let Some(image) = list.images.get_mut(index as usize) else {
            return Some(false);
        };
        *image = ImageListImage {
            bitmap,
            mask,
            icon: 0,
            transparent_color: None,
            source_x: 0,
            source_y: 0,
        };
        Some(true)
    }

    pub fn replace_image_list_icon(&mut self, handle: u32, index: i32, icon: u32) -> Option<i32> {
        if index < -1 || icon == 0 {
            return None;
        }
        let list = self.image_lists.get_mut(&handle)?;
        if index < 0 {
            let index = list.images.len();
            list.images.push(ImageListImage {
                bitmap: 0,
                mask: 0,
                icon,
                transparent_color: None,
                source_x: 0,
                source_y: 0,
            });
            return i32::try_from(index).ok();
        }
        let image = list.images.get_mut(index as usize)?;
        *image = ImageListImage {
            bitmap: 0,
            mask: 0,
            icon,
            transparent_color: None,
            source_x: 0,
            source_y: 0,
        };
        Some(index)
    }

    pub fn remove_image_list_image(&mut self, handle: u32, index: i32) -> Option<bool> {
        let list = self.image_lists.get_mut(&handle)?;
        if index == -1 {
            list.images.clear();
            list.overlays.clear();
            return Some(true);
        }
        if index < 0 {
            return Some(false);
        }
        let index = index as usize;
        if index >= list.images.len() {
            return Some(false);
        }
        remove_image_list_image_at(list, index);
        Some(true)
    }

    pub fn copy_image_list_image(
        &mut self,
        dst_handle: u32,
        dst_index: i32,
        src_handle: u32,
        src_index: i32,
        flags: u32,
    ) -> Option<bool> {
        const ILCF_SWAP: u32 = 0x0000_0001;
        if flags & !ILCF_SWAP != 0 || dst_index < 0 || src_index < 0 {
            return Some(false);
        }
        if !self.image_lists.contains_key(&dst_handle)
            || !self.image_lists.contains_key(&src_handle)
        {
            return None;
        }
        if dst_handle != src_handle {
            return Some(false);
        }
        let list = self.image_lists.get_mut(&dst_handle)?;
        let dst_index = dst_index as usize;
        let src_index = src_index as usize;
        if dst_index >= list.images.len() || src_index >= list.images.len() {
            return Some(false);
        }
        if flags & ILCF_SWAP != 0 {
            list.images.swap(dst_index, src_index);
            return Some(true);
        }
        let image = list.images[src_index].clone();
        list.images[dst_index] = image;
        Some(true)
    }

    pub fn copy_dither_image_list_image(
        &mut self,
        dst_handle: u32,
        dst_index: i32,
        x: i32,
        y: i32,
        src_handle: u32,
        src_index: i32,
        flags: u32,
    ) -> Option<bool> {
        const ILD_OVERLAYMASK: u32 = 0x0000_0f00;
        if dst_index < 0 || src_index < 0 {
            return Some(false);
        }
        let Some(src) = self.image_lists.get(&src_handle) else {
            return None;
        };
        if src.images.get(src_index as usize).is_none() {
            return Some(false);
        }
        let Some(dst) = self.image_lists.get_mut(&dst_handle) else {
            return None;
        };
        if dst.images.get(dst_index as usize).is_none() {
            return Some(false);
        }
        dst.last_dither_copy = Some(ImageListDitherCopy {
            dst_image_list: dst_handle,
            dst_index,
            x,
            y,
            src_image_list: src_handle,
            src_index,
            flags: flags & ILD_OVERLAYMASK,
        });
        Some(true)
    }

    pub fn set_image_list_count(&mut self, handle: u32, count: u32) -> Option<bool> {
        let list = self.image_lists.get_mut(&handle)?;
        let count = usize::try_from(count).ok()?;
        if count > MAX_EMULATED_IMAGE_LIST_IMAGES {
            return Some(false);
        }
        if count > list.images.len() {
            if list
                .images
                .try_reserve_exact(count - list.images.len())
                .is_err()
            {
                return Some(false);
            }
            list.images.resize(count, ImageListImage::default());
        } else {
            list.images.truncate(count);
        }
        Some(true)
    }

    pub fn image_list_count(&self, handle: u32) -> Option<usize> {
        Some(self.image_lists.get(&handle)?.images.len())
    }

    pub fn image_list_icon(
        &self,
        handle: u32,
        index: i32,
        fallback_icon: u32,
        flags: u32,
    ) -> Option<u32> {
        const ILD_OVERLAYMASK: u32 = 0x0000_0f00;
        if index < 0 {
            return None;
        }
        let list = self.image_lists.get(&handle)?;
        let image = list.images.get(index as usize)?;
        let base_icon = if image.icon != 0 {
            Some(image.icon)
        } else if fallback_icon != 0 {
            Some(fallback_icon)
        } else if image.bitmap != 0 {
            Some(0x000b_8000 | (image.bitmap & 0x0000_ffff))
        } else {
            Some(0)
        }?;
        let overlay = (flags & ILD_OVERLAYMASK) >> 8;
        if overlay == 0 {
            return Some(base_icon);
        }
        Some(match list.overlays.get(&overlay) {
            Some(overlay) => overlay_icon_handle(base_icon, overlay.image_index),
            None => base_icon,
        })
    }

    pub fn image_list_info(&self, handle: u32, index: i32) -> Option<ImageListImageInfo> {
        if index < 0 {
            return None;
        }
        let list = self.image_lists.get(&handle)?;
        let image = list.images.get(index as usize)?;
        let left = image.source_x;
        Some(ImageListImageInfo {
            bitmap: image.bitmap,
            mask: image.mask,
            left,
            top: image.source_y,
            right: left.saturating_add(list.width),
            bottom: image.source_y.saturating_add(list.height),
        })
    }

    pub fn set_image_list_bk_color(&mut self, handle: u32, color: u32) -> Option<u32> {
        let list = self.image_lists.get_mut(&handle)?;
        let previous = list.bk_color;
        list.bk_color = color;
        Some(previous)
    }

    pub fn set_image_list_overlay(
        &mut self,
        handle: u32,
        image_index: i32,
        overlay: i32,
    ) -> Option<bool> {
        const NUM_OVERLAY_IMAGES: i32 = 4;
        if image_index < 0 || !(1..=NUM_OVERLAY_IMAGES).contains(&overlay) {
            return Some(false);
        }
        let list = self.image_lists.get_mut(&handle)?;
        if list.flags & ILC_MASK == 0 {
            return Some(false);
        }
        if image_index as usize >= list.images.len() {
            return Some(false);
        }
        if list
            .overlays
            .get(&(overlay as u32))
            .is_some_and(|record| record.image_index == image_index)
        {
            return Some(true);
        }
        list.overlays.insert(
            overlay as u32,
            ImageListOverlay {
                image_index,
                x: 0,
                y: 0,
                width: list.width,
                height: list.height,
                flags: 0,
            },
        );
        Some(true)
    }

    pub fn set_image_list_overlay_bounds(
        &mut self,
        handle: u32,
        overlay: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        flags: u32,
    ) -> Option<bool> {
        let list = self.image_lists.get_mut(&handle)?;
        let Some(record) = list.overlays.get_mut(&(overlay as u32)) else {
            return Some(false);
        };
        record.x = x;
        record.y = y;
        record.width = width;
        record.height = height;
        record.flags = flags;
        Some(true)
    }

    pub fn record_image_list_draw(&mut self, mut draw: ImageListDraw) -> Option<bool> {
        const ILD_OVERLAYMASK: u32 = 0x0000_0f00;
        let list = self.image_lists.get_mut(&draw.image_list)?;
        if draw.index < 0 || draw.index as usize >= list.images.len() {
            return Some(false);
        }
        let overlay = (draw.flags & ILD_OVERLAYMASK) >> 8;
        draw.overlay_image = if overlay == 0 {
            None
        } else {
            list.overlays
                .get(&overlay)
                .map(|overlay| overlay.image_index)
        };
        list.last_draw = Some(draw);
        Some(true)
    }

    pub fn begin_image_list_drag(
        &mut self,
        handle: u32,
        index: i32,
        hotspot_x: i32,
        hotspot_y: i32,
    ) -> Option<bool> {
        if !self.image_list_has_index(handle, index) {
            return Some(false);
        }
        self.image_list_drag_hotspot_x = hotspot_x;
        self.image_list_drag_hotspot_y = hotspot_y;
        self.image_list_drag = Some(ImageListDragState {
            image_list: handle,
            index,
            hotspot_x,
            hotspot_y,
            lock_hwnd: self.image_list_drag_lock_hwnd,
            x: self.image_list_drag_x,
            y: self.image_list_drag_y,
            visible: false,
        });
        Some(true)
    }

    pub fn set_image_list_drag_cursor(
        &mut self,
        handle: u32,
        index: i32,
        hotspot_x: i32,
        hotspot_y: i32,
    ) -> Option<bool> {
        self.image_lists.get(&handle)?;
        if self.image_list_drag.is_none() {
            return Some(true);
        }
        if !self.image_list_has_index(handle, index) {
            return Some(false);
        }
        let drag = self.image_list_drag.as_mut()?;
        drag.image_list = handle;
        drag.index = index;
        drag.hotspot_x = hotspot_x;
        drag.hotspot_y = hotspot_y;
        Some(true)
    }

    pub fn image_list_drag_enter(&mut self, hwnd: u32, x: i32, y: i32) -> bool {
        if self.image_list_drag_lock_hwnd != 0 {
            return false;
        }
        self.image_list_drag_lock_hwnd = hwnd;
        self.image_list_drag_x = x;
        self.image_list_drag_y = y;
        if let Some(drag) = self.image_list_drag.as_mut() {
            drag.lock_hwnd = hwnd;
            drag.x = x;
            drag.y = y;
            drag.visible = true;
        }
        true
    }

    pub fn image_list_drag_move(&mut self, x: i32, y: i32) -> bool {
        if let Some(drag) = self.image_list_drag.as_mut() {
            if drag.visible {
                drag.x = x;
                drag.y = y;
                self.image_list_drag_x = x;
                self.image_list_drag_y = y;
            }
        }
        true
    }

    pub fn image_list_drag_leave(&mut self, hwnd: u32) -> bool {
        if self.image_list_drag_lock_hwnd != hwnd {
            return false;
        }
        self.image_list_drag_lock_hwnd = 0;
        if let Some(drag) = self.image_list_drag.as_mut() {
            drag.lock_hwnd = 0;
            drag.visible = false;
        }
        true
    }

    pub fn image_list_drag_show(&mut self, visible: bool) -> bool {
        let Some(drag) = self.image_list_drag.as_mut() else {
            return false;
        };
        drag.visible = visible;
        true
    }

    pub fn end_image_list_drag(&mut self) -> bool {
        self.image_list_drag.take().is_some()
    }

    pub fn image_list_drag(&self) -> Option<ImageListDragState> {
        self.image_list_drag
    }

    pub fn image_list_drag_position(&self) -> (i32, i32) {
        (self.image_list_drag_x, self.image_list_drag_y)
    }

    pub fn image_list_drag_hotspot(&self) -> (i32, i32) {
        (
            self.image_list_drag_hotspot_x,
            self.image_list_drag_hotspot_y,
        )
    }

    fn image_list_has_index(&self, handle: u32, index: i32) -> bool {
        index >= 0
            && self
                .image_lists
                .get(&handle)
                .is_some_and(|list| (index as usize) < list.images.len())
    }

    pub fn create_font(
        &mut self,
        logfont_ptr: u32,
        height: i32,
        width: i32,
        escapement: i32,
        orientation: i32,
        weight: i32,
        italic: bool,
        underline: bool,
        strikeout: bool,
        charset: u8,
        out_precision: u8,
        clip_precision: u8,
        quality: u8,
        pitch_and_family: u8,
        face_name: String,
    ) -> u32 {
        let handle = self.next_gdi_handle;
        self.next_gdi_handle += 4;
        self.fonts.insert(
            handle,
            FontObject {
                handle,
                logfont_ptr,
                height,
                width,
                escapement,
                orientation,
                weight,
                italic,
                underline,
                strikeout,
                charset,
                out_precision,
                clip_precision,
                quality,
                pitch_and_family,
                face_name,
            },
        );
        handle
    }

    pub fn font(&self, handle: u32) -> Option<&FontObject> {
        self.fonts.get(&handle)
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
                owns_pattern_bitmap: false,
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
                owns_pattern_bitmap: false,
            },
        );
        Some(handle)
    }

    pub fn create_owned_pattern_brush(&mut self, bitmap: u32) -> Option<u32> {
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
                owns_pattern_bitmap: true,
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

    pub fn gdi_object_kind(&self, handle: u32) -> &'static str {
        if self.bitmaps.contains_key(&handle) || is_default_bitmap(handle) {
            "bitmap"
        } else if self.fonts.contains_key(&handle) || is_stock_font(handle) {
            "font"
        } else if self.brushes.contains_key(&handle) || is_stock_brush(handle) {
            "brush"
        } else if self.pens.contains_key(&handle) || is_stock_pen(handle) {
            "pen"
        } else if self.palettes.contains_key(&handle) || is_stock_palette(handle) {
            "palette"
        } else if self.regions.contains_key(&handle) {
            "region"
        } else if self.image_lists.contains_key(&handle) {
            "image_list"
        } else if self.memory_dcs.contains(&handle) {
            "memory_dc"
        } else {
            "unknown"
        }
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
        let is_bitmap = self.bitmaps.contains_key(&object) || is_default_bitmap(object);
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

    pub fn get_current_object(&self, hdc: u32, obj_type: u32) -> u32 {
        self.dc_state(hdc)
            .map(|s| match obj_type {
                1 => s.selected_pen,     // OBJ_PEN
                2 => s.selected_brush,   // OBJ_BRUSH
                5 => s.selected_palette, // OBJ_PAL
                6 => s.selected_font,    // OBJ_FONT
                7 => s.selected_bitmap,  // OBJ_BITMAP
                _ => 0,
            })
            .unwrap_or(0)
    }

    pub fn stretch_blt_mode(&self, hdc: u32) -> i32 {
        self.dc_states
            .get(&hdc)
            .map(|s| s.stretch_blt_mode)
            .unwrap_or(1)
    }

    pub fn set_stretch_blt_mode(&mut self, hdc: u32, mode: i32) -> i32 {
        if let Some(s) = self.dc_states.get_mut(&hdc) {
            let old = s.stretch_blt_mode;
            s.stretch_blt_mode = mode;
            old
        } else {
            0
        }
    }

    pub fn text_char_extra(&self, hdc: u32) -> i32 {
        self.dc_states
            .get(&hdc)
            .map(|s| s.text_char_extra)
            .unwrap_or(0)
    }

    pub fn set_text_char_extra(&mut self, hdc: u32, extra: i32) -> i32 {
        if let Some(s) = self.dc_states.get_mut(&hdc) {
            let old = s.text_char_extra;
            s.text_char_extra = extra;
            old
        } else {
            0
        }
    }

    pub fn layout(&self, hdc: u32) -> u32 {
        self.dc_states.get(&hdc).map(|s| s.layout).unwrap_or(0)
    }

    pub fn set_layout(&mut self, hdc: u32, layout: u32) -> u32 {
        if let Some(s) = self.dc_states.get_mut(&hdc) {
            let old = s.layout;
            s.layout = layout;
            old
        } else {
            0xffff_ffff // GDI_ERROR
        }
    }

    pub fn selected_bitmap(&self, hdc: u32) -> Option<u32> {
        self.dc_states
            .get(&hdc)
            .map(|state| state.selected_bitmap)
            .filter(|bitmap| *bitmap != 0 && !is_default_bitmap(*bitmap))
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

    pub fn dc_state(&self, hdc: u32) -> Option<DcState> {
        if hdc == 0 {
            None
        } else {
            Some(self.dc_states.get(&hdc).cloned().unwrap_or_default())
        }
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

    pub fn set_brush_origin(
        &mut self,
        hdc: u32,
        point: crate::ce::gwe::Point,
    ) -> Option<crate::ce::gwe::Point> {
        if hdc == 0 {
            return None;
        }
        let state = self.dc_states.entry(hdc).or_default();
        let previous = state.brush_origin;
        state.brush_origin = point;
        Some(previous)
    }

    pub fn is_memory_dc(&self, hdc: u32) -> bool {
        self.memory_dcs.contains(&hdc)
    }

    pub fn select_palette(&mut self, hdc: u32, palette: u32) -> Option<u32> {
        if hdc == 0 || (!self.palettes.contains_key(&palette) && !is_stock_palette(palette)) {
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

    pub fn set_dc_rop2(&mut self, hdc: u32, rop2: i32) -> Option<i32> {
        if hdc == 0 {
            return None;
        }
        let state = self.dc_states.entry(hdc).or_default();
        let previous = state.rop2;
        state.rop2 = rop2;
        Some(previous)
    }

    pub fn get_dc_bk_color(&self, hdc: u32) -> Option<u32> {
        if hdc == 0 {
            return None;
        }
        Some(
            self.dc_states
                .get(&hdc)
                .map(|s| s.bk_color)
                .unwrap_or(0x00ff_ffff),
        )
    }

    pub fn get_dc_bk_mode(&self, hdc: u32) -> Option<i32> {
        if hdc == 0 {
            return None;
        }
        Some(self.dc_states.get(&hdc).map(|s| s.bk_mode).unwrap_or(2))
    }

    /// Push the current DC state onto the per-DC save stack.
    /// Returns the new save level (1-based) or 0 on failure.
    pub fn save_dc(&mut self, hdc: u32) -> i32 {
        if hdc == 0 {
            return 0;
        }
        let current = self.dc_states.entry(hdc).or_default().clone();
        let stack = self.dc_save_stacks.entry(hdc).or_default();
        stack.push(current);
        stack.len() as i32
    }

    /// Pop DC state from the save stack. `level` is the 1-based save level
    /// returned by `save_dc`, or a negative offset where -1 is the most recent
    /// save level.
    /// Returns true on success.
    pub fn restore_dc(&mut self, hdc: u32, level: i32) -> bool {
        if hdc == 0 || level == 0 {
            return false;
        }
        let stack = match self.dc_save_stacks.get_mut(&hdc) {
            Some(s) if !s.is_empty() => s,
            _ => return false,
        };
        let target_len = if level < 0 {
            let relative = stack.len() as i32 + level;
            if relative < 0 {
                return false;
            }
            relative as usize
        } else {
            (level as usize).saturating_sub(1)
        };
        if target_len >= stack.len() {
            return false;
        }
        stack.truncate(target_len + 1);
        let saved = stack.pop().unwrap();
        self.dc_states.insert(hdc, saved);
        true
    }

    pub fn delete_gdi_object(&mut self, handle: u32) -> bool {
        if self.is_gdi_object_selected(handle) {
            return false;
        }
        let mut removed = self.fonts.remove(&handle).is_some();
        removed |= self.brushes.remove(&handle).is_some();
        removed |= self.pens.remove(&handle).is_some();
        removed |= self.bitmaps.remove(&handle).is_some();
        removed |= self.delete_region(handle);
        removed |= self.image_lists.remove(&handle).is_some();
        removed |= self.palettes.remove(&handle).is_some();
        removed
    }

    fn is_gdi_object_selected(&self, handle: u32) -> bool {
        self.dc_states.values().any(|state| {
            state.selected_object == handle
                || state.selected_bitmap == handle
                || state.selected_font == handle
                || state.selected_brush == handle
                || state.selected_pen == handle
                || state.selected_palette == handle
        })
    }
}

fn popup_menu_item_is_selectable(item: &MenuItem) -> bool {
    item.submenu == 0 && item.id != 0 && item.id != u32::MAX && popup_menu_item_is_enabled(item)
}

fn popup_menu_item_is_enabled(item: &MenuItem) -> bool {
    item.item_type & MF_SEPARATOR == 0 && item.state & (MF_DISABLED | MF_GRAYED) == 0
}

fn menu_item_index(menu: &MenuObject, item_or_pos: u32, by_position: bool) -> Option<usize> {
    if by_position {
        let index = item_or_pos as usize;
        (index < menu.items.len()).then_some(index)
    } else {
        menu.items.iter().position(|item| item.id == item_or_pos)
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

    #[test]
    fn remove_module_resources_removes_only_resource_entries_and_strings_for_module() {
        let mut resources = ResourceSystem::default();
        let removed_handle = resources.register(
            0x0001_0000,
            ResourceId::Integer(10),
            ResourceId::Integer(3),
            0x0040_0000,
            16,
        );
        let kept_handle = resources.register(
            0x0002_0000,
            ResourceId::Integer(10),
            ResourceId::Integer(3),
            0x0050_0000,
            20,
        );
        resources.register_string(0x0001_0000, 7, "removed", Some(0x0040_0100));
        resources.register_string(0x0002_0000, 7, "kept", Some(0x0050_0100));

        assert_eq!(resources.remove_module_resources(0x0001_0000), 2);
        assert_eq!(resources.resource_entry(removed_handle), None);
        assert_eq!(
            resources.find_resource(0x0001_0000, ResourceId::Integer(10), ResourceId::Integer(3)),
            None
        );
        assert_eq!(resources.load_string(0x0001_0000, 7), None);
        assert!(resources.resource_entry(kept_handle).is_some());
        assert_eq!(
            resources.find_resource(0x0002_0000, ResourceId::Integer(10), ResourceId::Integer(3)),
            Some(kept_handle)
        );
        assert_eq!(
            resources
                .load_string(0x0002_0000, 7)
                .map(|value| value.text.as_str()),
            Some("kept")
        );
    }
}
