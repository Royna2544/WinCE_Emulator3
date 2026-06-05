use std::collections::BTreeMap;

use crate::{
    Error, Result,
    ce::framebuffer::{
        Framebuffer, FramebufferInfo, FramebufferRect, FramebufferSnapshot, VirtualFramebuffer,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Presentation {
    pub framebuffer: FramebufferSnapshot,
    pub dirty_rects: Vec<FramebufferRect>,
}

pub trait Presenter {
    fn present(&mut self, framebuffer: &dyn Framebuffer) -> Result<Presentation>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VirtualInputEvent {
    Key { virtual_key: u32, pressed: bool },
    TouchDown { x: i32, y: i32 },
    TouchMove { x: i32, y: i32 },
    TouchUp { x: i32, y: i32 },
}

pub trait Input {
    fn poll_events(&mut self) -> Result<Vec<VirtualInputEvent>>;
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VirtualInput {
    events: Vec<VirtualInputEvent>,
}

impl VirtualInput {
    pub fn push_event(&mut self, event: VirtualInputEvent) {
        self.events.push(event);
    }

    pub fn push_touch_down(&mut self, x: i32, y: i32) {
        self.push_event(VirtualInputEvent::TouchDown { x, y });
    }

    pub fn push_touch_move(&mut self, x: i32, y: i32) {
        self.push_event(VirtualInputEvent::TouchMove { x, y });
    }

    pub fn push_touch_up(&mut self, x: i32, y: i32) {
        self.push_event(VirtualInputEvent::TouchUp { x, y });
    }
}

impl Input for VirtualInput {
    fn poll_events(&mut self) -> Result<Vec<VirtualInputEvent>> {
        Ok(std::mem::take(&mut self.events))
    }
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
pub struct VirtualDesktop<I = VirtualInput, P = VirtualPresenter> {
    framebuffer: VirtualFramebuffer,
    input: I,
    presenter: P,
    next_id: u32,
    windows: BTreeMap<DesktopWindowId, DesktopWindow>,
}

impl VirtualDesktop {
    pub fn new(surface_info: FramebufferInfo) -> Self {
        let framebuffer = VirtualFramebuffer::with_stride(
            surface_info.width,
            surface_info.height,
            surface_info.stride,
            surface_info.format,
        )
        .expect("desktop surface info must describe a valid framebuffer");
        Self::with_parts(
            framebuffer,
            VirtualInput::default(),
            VirtualPresenter::default(),
        )
    }

    pub fn default_primary() -> Result<Self> {
        Ok(Self::with_parts(
            VirtualFramebuffer::default_primary()?,
            VirtualInput::default(),
            VirtualPresenter::default(),
        ))
    }
}

impl<I, P> VirtualDesktop<I, P> {
    pub fn with_parts(framebuffer: VirtualFramebuffer, input: I, presenter: P) -> Self {
        Self {
            framebuffer,
            input,
            presenter,
            next_id: 1,
            windows: BTreeMap::new(),
        }
    }

    pub fn framebuffer(&self) -> &VirtualFramebuffer {
        &self.framebuffer
    }

    pub fn framebuffer_mut(&mut self) -> &mut VirtualFramebuffer {
        &mut self.framebuffer
    }

    pub fn input(&self) -> &I {
        &self.input
    }

    pub fn input_mut(&mut self) -> &mut I {
        &mut self.input
    }

    pub fn presenter(&self) -> &P {
        &self.presenter
    }

    pub fn presenter_mut(&mut self) -> &mut P {
        &mut self.presenter
    }

    pub fn framebuffer_and_presenter_mut(&mut self) -> (&mut VirtualFramebuffer, &mut P) {
        (&mut self.framebuffer, &mut self.presenter)
    }
}

impl<I: Input, P> VirtualDesktop<I, P> {
    pub fn poll_input(&mut self) -> Result<Vec<VirtualInputEvent>> {
        self.input.poll_events()
    }
}

impl<I, P: Presenter> VirtualDesktop<I, P> {
    pub fn present(&mut self) -> Result<Presentation> {
        self.presenter.present(&self.framebuffer)
    }
}

impl<I, P> Desktop for VirtualDesktop<I, P> {
    fn surface_info(&self) -> FramebufferInfo {
        self.framebuffer.info()
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
