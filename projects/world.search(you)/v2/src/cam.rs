//! Virtual camera: a post-transform that pans, pushes in and shakes the
//! finished frame. Every scene drives one; nothing else touches pixels.

use crate::gfx::Rng;
use crate::pix::{Canvas, H, W};

#[derive(Clone, Copy, Debug)]
pub struct Cam {
    /// >1 pushes in, <1 pulls out
    pub zoom: f32,
    /// pan in pixels (positive = the world moves right / down)
    pub x: f32,
    pub y: f32,
    /// handheld shake amplitude in pixels
    pub shake: f32,
    /// focus point in pixels (what the zoom is centred on)
    pub fx: f32,
    pub fy: f32,
}

impl Default for Cam {
    fn default() -> Self {
        Cam { zoom: 1.0, x: 0.0, y: 0.0, shake: 0.0, fx: W as f32 / 2.0, fy: H as f32 / 2.0 }
    }
}

impl Cam {
    pub fn new() -> Cam {
        Cam::default()
    }

    /// Frame the whole screen with a static camera.
    pub fn still() -> Cam {
        Cam::default()
    }

    /// Coarse scene shake for impacts; decays with `energy`.
    pub fn with_shake(mut self, amp: f32) -> Cam {
        self.shake = amp;
        self
    }
}

/// Resample the frame through the camera. Bilinear, deterministic.
pub fn apply(cv: &mut Canvas, cam: &Cam, t: f64) {
    let shake = cam.shake;
    let (sx, sy) = if shake > 0.01 {
        let mut rng = Rng::new(((t * 137.0) as u64).wrapping_mul(0x9E3779B97F4A7C15) | 1);
        (
            (rng.f64() * 2.0 - 1.0) as f32 * shake,
            (rng.f64() * 2.0 - 1.0) as f32 * shake,
        )
    } else {
        (0.0, 0.0)
    };

    let zoom = cam.zoom.clamp(0.35, 4.0);
    let off_x = cam.x + sx;
    let off_y = cam.y + sy;

    if (zoom - 1.0).abs() < 1e-4 && off_x.abs() < 0.01 && off_y.abs() < 0.01 {
        return;
    }

    let src = cv.buf.clone();
    for y in 0..H {
        let dy = y as f32 + 0.5 - cam.fy;
        let syf = cam.fy + (dy - off_y) / zoom - 0.5;
        let y0 = syf.floor();
        let fy = (syf - y0).clamp(0.0, 1.0);
        let y0 = y0 as i32;
        let ya = y0.clamp(0, H as i32 - 1) as usize;
        let yb = (y0 + 1).clamp(0, H as i32 - 1) as usize;
        for x in 0..W {
            let dx = x as f32 + 0.5 - cam.fx;
            let sxf = cam.fx + (dx - off_x) / zoom - 0.5;
            let x0 = sxf.floor();
            let fx = (sxf - x0).clamp(0.0, 1.0);
            let x0 = x0 as i32;
            let xa = x0.clamp(0, W as i32 - 1) as usize;
            let xb = (x0 + 1).clamp(0, W as i32 - 1) as usize;

            let ia = (ya * W + xa) * 3;
            let ib = (ya * W + xb) * 3;
            let ic = (yb * W + xa) * 3;
            let id = (yb * W + xb) * 3;
            let di = (y * W + x) * 3;
            for c in 0..3 {
                let top = src[ia + c] as f32 * (1.0 - fx) + src[ib + c] as f32 * fx;
                let bot = src[ic + c] as f32 * (1.0 - fx) + src[id + c] as f32 * fx;
                let v = top * (1.0 - fy) + bot * fy;
                cv.buf[di + c] = v.clamp(0.0, 255.0) as u8;
            }
        }
    }
}
