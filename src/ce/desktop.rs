use std::collections::BTreeMap;

use crate::{
    Error, Result,
    ce::framebuffer::{Framebuffer, FramebufferInfo, FramebufferRect, FramebufferSnapshot},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Presentation {
    pub framebuffer: FramebufferSnapshot,
    pub dirty_rects: Vec<FramebufferRect>,
}

pub trait Presenter {
    fn present(&mut self, framebuffer: &dyn Framebuffer) -> Result<Presentation>;
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VirtualPresenter {
    last_presentation: Option<Presentation>,
    presentation_count: u64,
}

impl VirtualPresenter {
    pub fn last_presentation(&self) -> Option<&Presentation> {
        self.last_presentation.as_ref()
    }

    pub fn presentation_count(&self) -> u64 {
        self.presentation_count
    }
}

impl Presenter for VirtualPresenter {
    fn present(&mut self, framebuffer: &dyn Framebuffer) -> Result<Presentation> {
        let snapshot = FramebufferSnapshot {
            info: framebuffer.info(),
            pixels: framebuffer.pixels().to_vec(),
        };
        let presentation = Presentation {
            framebuffer: snapshot,
            dirty_rects: framebuffer.dirty_rects().to_vec(),
        };
        self.last_presentation = Some(presentation.clone());
        self.presentation_count = self.presentation_count.saturating_add(1);
        Ok(presentation)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DesktopWindowId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopWindowSpec {
    pub title: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub visible: bool,
}

impl DesktopWindowSpec {
    pub fn new(title: impl Into<String>, x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            title: title.into(),
            x,
            y,
            width,
            height,
            visible: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopWindow {
    pub id: DesktopWindowId,
    pub title: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub visible: bool,
}

impl DesktopWindow {
    fn from_spec(id: DesktopWindowId, spec: DesktopWindowSpec) -> Self {
        Self {
            id,
            title: spec.title,
            x: spec.x,
            y: spec.y,
            width: spec.width,
            height: spec.height,
            visible: spec.visible,
        }
    }
}

pub trait Desktop {
    fn surface_info(&self) -> FramebufferInfo;
    fn create_window(&mut self, spec: DesktopWindowSpec) -> Result<DesktopWindowId>;
    fn move_window(
        &mut self,
        id: DesktopWindowId,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> Result<()>;
    fn remove_window(&mut self, id: DesktopWindowId) -> Result<DesktopWindow>;
    fn window(&self, id: DesktopWindowId) -> Option<&DesktopWindow>;
    fn windows(&self) -> Vec<&DesktopWindow>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtualDesktop {
    surface_info: FramebufferInfo,
    next_id: u32,
    windows: BTreeMap<DesktopWindowId, DesktopWindow>,
}

impl VirtualDesktop {
    pub fn new(surface_info: FramebufferInfo) -> Self {
        Self {
            surface_info,
            next_id: 1,
            windows: BTreeMap::new(),
        }
    }
}

impl Desktop for VirtualDesktop {
    fn surface_info(&self) -> FramebufferInfo {
        self.surface_info
    }

    fn create_window(&mut self, spec: DesktopWindowSpec) -> Result<DesktopWindowId> {
        if spec.width == 0 || spec.height == 0 {
            return Err(Error::InvalidArgument(
                "desktop window dimensions must be nonzero".to_owned(),
            ));
        }
        let id = DesktopWindowId(self.next_id);
        self.next_id = self.next_id.saturating_add(1).max(1);
        self.windows.insert(id, DesktopWindow::from_spec(id, spec));
        Ok(id)
    }

    fn move_window(
        &mut self,
        id: DesktopWindowId,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> Result<()> {
        if width == 0 || height == 0 {
            return Err(Error::InvalidArgument(
                "desktop window dimensions must be nonzero".to_owned(),
            ));
        }
        let Some(window) = self.windows.get_mut(&id) else {
            return Err(Error::InvalidArgument(format!(
                "unknown desktop window {}",
                id.0
            )));
        };
        window.x = x;
        window.y = y;
        window.width = width;
        window.height = height;
        Ok(())
    }

    fn remove_window(&mut self, id: DesktopWindowId) -> Result<DesktopWindow> {
        self.windows
            .remove(&id)
            .ok_or_else(|| Error::InvalidArgument(format!("unknown desktop window {}", id.0)))
    }

    fn window(&self, id: DesktopWindowId) -> Option<&DesktopWindow> {
        self.windows.get(&id)
    }

    fn windows(&self) -> Vec<&DesktopWindow> {
        self.windows.values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ce::framebuffer::{Framebuffer, PixelFormat, VirtualFramebuffer};

    #[test]
    fn virtual_presenter_snapshots_trait_framebuffer() {
        let mut framebuffer = VirtualFramebuffer::new(4, 3, PixelFormat::Gray8).unwrap();
        framebuffer.clear(0x7f);

        let mut presenter = VirtualPresenter::default();
        let presentation = presenter.present(&framebuffer).unwrap();

        assert_eq!(presenter.presentation_count(), 1);
        assert_eq!(presentation.framebuffer.info.width, 4);
        assert_eq!(presentation.framebuffer.info.height, 3);
        assert_eq!(presentation.framebuffer.pixels, vec![0x7f; 12]);
        assert_eq!(presentation.dirty_rects, framebuffer.dirty_rects());
        assert_eq!(
            presenter.last_presentation().unwrap().framebuffer.info,
            framebuffer.info()
        );
    }

    #[test]
    fn virtual_desktop_manages_window_lifecycle() {
        let framebuffer = VirtualFramebuffer::new(320, 240, PixelFormat::Rgb565).unwrap();
        let mut desktop = VirtualDesktop::new(framebuffer.info());

        let id = desktop
            .create_window(DesktopWindowSpec::new("main", 10, 20, 100, 80))
            .unwrap();
        desktop.move_window(id, 30, 40, 120, 90).unwrap();

        let window = desktop.window(id).unwrap();
        assert_eq!(window.title, "main");
        assert_eq!(
            (window.x, window.y, window.width, window.height),
            (30, 40, 120, 90)
        );
        assert_eq!(desktop.windows().len(), 1);

        let removed = desktop.remove_window(id).unwrap();
        assert_eq!(removed.id, id);
        assert!(desktop.window(id).is_none());
    }
}
