use core::cmp::{max, min};
use core::ptr::write_volatile;

use crate::multiboot::FramebufferInfo;

pub struct Framebuffer {
    ptr: *mut u8,
    info: FramebufferInfo,
    bytes_per_pixel: usize,
}

impl Framebuffer {
    pub fn new(info: FramebufferInfo) -> Option<Self> {
        if info.address == 0 || info.width == 0 || info.height == 0 {
            return None;
        }

        let bytes_per_pixel = usize::from((info.bpp.saturating_add(7)) / 8);
        if !(3..=4).contains(&bytes_per_pixel) {
            return None;
        }

        Some(Self {
            ptr: info.address as *mut u8,
            info,
            bytes_per_pixel,
        })
    }

    pub fn width(&self) -> usize {
        self.info.width as usize
    }

    pub fn height(&self) -> usize {
        self.info.height as usize
    }

    pub fn fill_screen(&mut self, r: u8, g: u8, b: u8) {
        self.fill_rect(0, 0, self.width(), self.height(), r, g, b);
    }

    pub fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize, r: u8, g: u8, b: u8) {
        if w == 0 || h == 0 {
            return;
        }

        let x0 = min(x, self.width());
        let y0 = min(y, self.height());
        let x1 = min(x0.saturating_add(w), self.width());
        let y1 = min(y0.saturating_add(h), self.height());
        if x0 >= x1 || y0 >= y1 {
            return;
        }

        let packed_color = self.pack_color(r, g, b);
        for py in y0..y1 {
            for px in x0..x1 {
                self.put_pixel(px, py, packed_color);
            }
        }
    }

    fn put_pixel(&mut self, x: usize, y: usize, color: u32) {
        let offset = y
            .saturating_mul(self.info.pitch as usize)
            .saturating_add(x.saturating_mul(self.bytes_per_pixel));
        let ptr = unsafe { self.ptr.add(offset) };

        if self.bytes_per_pixel == 4 {
            unsafe {
                write_volatile(ptr as *mut u32, color);
            }
        } else {
            unsafe {
                write_volatile(ptr, (color & 0xFF) as u8);
                write_volatile(ptr.add(1), ((color >> 8) & 0xFF) as u8);
                write_volatile(ptr.add(2), ((color >> 16) & 0xFF) as u8);
            }
        }
    }

    fn pack_color(&self, r: u8, g: u8, b: u8) -> u32 {
        if self.info.buffer_type != 1 {
            return (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b);
        }

        pack_channel(r, self.info.red_field_position, self.info.red_mask_size)
            | pack_channel(g, self.info.green_field_position, self.info.green_mask_size)
            | pack_channel(b, self.info.blue_field_position, self.info.blue_mask_size)
    }
}

fn pack_channel(value: u8, position: u8, mask_size: u8) -> u32 {
    if mask_size == 0 || mask_size > 8 {
        return 0;
    }

    let max_channel = (1u32 << mask_size) - 1;
    let scaled = (u32::from(value) * max_channel) / 255;
    scaled << position
}

pub fn draw_desktop(framebuffer: &mut Framebuffer, tick: u64) {
    let width = framebuffer.width();
    let height = framebuffer.height();

    framebuffer.fill_screen(16, 24, 36);

    let top_bar_height = min(38, max(24, height / 16));
    framebuffer.fill_rect(0, 0, width, top_bar_height, 36, 59, 85);
    framebuffer.fill_rect(0, top_bar_height.saturating_sub(2), width, 2, 104, 145, 184);

    let panel_margin = min(width / 10, 48);
    let panel_x = panel_margin;
    let panel_y = top_bar_height + 26;
    let panel_w = width.saturating_sub(panel_margin * 2);
    let panel_h = height.saturating_sub(panel_y + panel_margin);
    framebuffer.fill_rect(panel_x, panel_y, panel_w, panel_h, 24, 38, 56);
    framebuffer.fill_rect(panel_x, panel_y, panel_w, 2, 90, 132, 173);

    let dock_h = min(72, max(46, height / 10));
    framebuffer.fill_rect(0, height.saturating_sub(dock_h), width, dock_h, 28, 45, 66);
    framebuffer.fill_rect(0, height.saturating_sub(dock_h), width, 2, 88, 124, 160);

    let dot_size = min(18, max(10, width / 96));
    let dot_gap = dot_size + 8;
    let dot_y = height.saturating_sub(dock_h / 2 + dot_size / 2);
    let dot_start_x = max(24, (width / 2).saturating_sub(dot_gap * 2));
    for i in 0..5 {
        let x = dot_start_x + i * dot_gap;
        let active = ((tick / 15) as usize) % 5 == i;
        if active {
            framebuffer.fill_rect(x, dot_y, dot_size, dot_size, 98, 208, 152);
        } else {
            framebuffer.fill_rect(x, dot_y, dot_size, dot_size, 73, 98, 126);
        }
    }

    let bar_w = min(panel_w.saturating_sub(40), max(120, width / 3));
    let bar_h = 20;
    let bar_x = panel_x + 20;
    let bar_y = panel_y + panel_h.saturating_sub(40);
    framebuffer.fill_rect(bar_x, bar_y, bar_w, bar_h, 45, 66, 89);
    framebuffer.fill_rect(bar_x, bar_y, bar_w, 2, 77, 114, 146);

    let phase = (tick % 200) as usize;
    let fill = (bar_w.saturating_sub(4) * phase) / 199;
    framebuffer.fill_rect(
        bar_x + 2,
        bar_y + 2,
        fill,
        bar_h.saturating_sub(4),
        86,
        198,
        132,
    );
}
