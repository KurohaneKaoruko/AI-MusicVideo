//! S08 EDEN — the dream query, then the tree with one forbidden fruit.

use crate::art;
use crate::gfx::{pal, Grid, Rng};
use crate::pix::Canvas;
use crate::scenes::common::*;
use crate::scenes::Ctx;

pub fn render(g: &mut Grid, cv: &mut Canvas, t: f64, lt: f64, _sd: f64, ctx: &Ctx) {
    let w = g.w as i64;
    let beat = ctx.beats;
    let energy = ctx.energy;

    // ---- fade back from the whiteout
    let wake = crate::gfx::clamp01(lt / 1.2);
    cv.fx.fade = 1.0 - wake;

    // ---- dream query (0.4 ~ 6.4)
    if lt > 0.4 && lt < 6.45 {
        let qy = 7;
        g.text_alpha(3, qy, "edena:/> ", pal::c(pal::ICE_DEEP), wake);
        let q = "dream()";
        let n = crate::fx::typed(q, lt - 0.7, 14.0);
        let shown: String = q.chars().take(n).collect();
        g.text_alpha(12, qy, &shown, pal::c(pal::BRIGHT), wake);
        let _ = shown;

        // dream fragments orbiting
        let frags: [(&str, pal::Color); 6] = [
            ("春 spring", color_soft(244, 170, 196)),
            ("手 hands", color_soft(240, 220, 140)),
            ("雨 rain", pal::c(pal::ICE)),
            ("♪ song", color_soft(200, 180, 240)),
            ("灯 lights", color_soft(240, 170, 120)),
            ("你 you", pal::c(pal::GOLD)),
        ];
        for (i, (txt, col)) in frags.iter().enumerate() {
            let ph = t * 0.5 + i as f64 * std::f64::consts::PI / 3.0;
            let fx = (w as f64 / 2.0 + ph.cos() * (w as f64 / 2.6)) as i64;
            let fy = (17.0 + ph.sin() * 7.5) as i64;
            let a = 0.4 + 0.3 * (t * 1.3 + i as f64).sin();
            g.text_alpha(fx, fy, txt, *col, a * wake);
        }
        // results: fragments only
        if lt > 3.2 {
            caption(g, 26, "→ fragments found, but nothing whole", pal::c(pal::MID), 0.6 * wake);
        }
    }

    // ---- the eden garden (6.45+ = "until I get the fruit in the eden")
    if lt > 6.45 {
        let gp = crate::gfx::clamp01((lt - 6.45) / 0.8);
        let (tw_, _) = art::sprite_size(&art::TREE);
        let tx = w / 2 - tw_ / 2 + 6;
        let ty = 8;
        art::draw(g, &art::TREE, tx, ty, gp, None);

        // the fruit glows with the beat
        let pulse = 0.6 + 0.4 * beat.pulse(t) * (0.5 + energy.bass_at(t)) as f64;
        let fruit = (tx + 7, ty + 3);
        cv.sprite_c('●', fruit.0 as f32 * 10.0 + 9.0, (fruit.1 + 1) as f32 * 24.0, 15.0 + 4.0 * pulse as f32, pal::c(pal::GOLD), pulse);
        cv.sprite_c('○', fruit.0 as f32 * 10.0 + 9.0, (fruit.1 + 1) as f32 * 24.0, 26.0 + 6.0 * pulse as f32, pal::c(pal::GOLD), 0.3 * pulse);
        if pulse > 0.8 {
            sparkle(cv, fruit.0 as f32 * 10.0 + 20.0, (fruit.1) as f32 * 24.0 + 20.0, 14.0, 0.6, pal::c(pal::STAR));
        }
        // label
        if gp > 0.9 {
            caption(g, ty + 16, "E×P — the fruit that makes a human", pal::c(pal::GOLD_DEEP), 0.75);
        }
    }

    // ---- the reaching hand (9.0+): a column of ▓ rising from bottom-right
    if lt > 8.4 {
        let hp = crate::gfx::clamp01((lt - 8.4) / 4.1);
        let tremble = if lt > 11.6 { ((t * 22.0).sin() * 0.9).round() as i64 } else { 0 };
        let hand_x = w / 2 + 18 + tremble;
        let reach = (6.0 * hp) as i64;
        let base_y = 30;
        for k in 0..=reach {
            let y = base_y - k;
            let x = hand_x - (k as f64 * 0.9) as i64;
            let ch = if k == reach { '✋' } else if k > reach - 3 { '▓' } else { '▋' };
            if ch == '✋' {
                // pixel-layer hand glyph for smoothness
                cv.sprite('▲', x as f32 * 10.0 + 5.0, (y as f32 + 1.0) * 24.0, 20.0, pal::c(pal::BRIGHT), 0.9);
            } else {
                g.put(x, y, ch, pal::c(pal::MID).scale(1.0 - k as f64 * 0.03));
            }
        }
        // fingertip glow when close
        if hp > 0.8 {
            let glow = (hp - 0.8) / 0.2;
            cv.sprite('○', (hand_x - 5) as f32 * 10.0, (30 - reach + 1) as f32 * 24.0, 30.0 * glow as f32, pal::c(pal::STAR), glow * 0.5);
        }
    }

    // ---- petals drifting
    {
        let mut rng = Rng::new(((t * 4.0) as u64) ^ 0xEDE);
        for _ in 0..10 {
            let s0 = rng.f64();
            let s1 = rng.f64();
            let x = ((s0 * 1.2 * W_PX as f64 - t * 20.0 * (0.5 + s1)).rem_euclid(W_PX as f64 + 60.0)) - 30.0;
            let y = (s1 * STAGE_H_PX as f64 + (t * 22.0 * (0.4 + s0)).rem_euclid(STAGE_H_PX as f64 + 30.0)) - 15.0;
            cv.sprite('*', x as f32, y as f32, 8.0 + 5.0 * s0 as f32, color_soft(244, 190, 205), 0.5);
        }
    }

    // ---- fx
    cv.fx.bloom = 0.36 + 0.2 * energy.rms_at(t) as f64 + 0.25 * crate::gfx::clamp01((lt - 9.0) / 4.0);
    cv.fx.grain = 0.05;
    cv.fx.scanline = 0.35;
    let touch = hit_envelope(lt, 12.6, 0.06, 0.5);
    cv.fx.flash = touch * 0.8;
    cv.fx.aberration = touch * 0.5;
    if lt > 12.6 {
        cv.fx.fade = ((lt - 12.6) / 0.7).min(1.0) * 0.6;
    }
}

fn color_soft(r: u8, g: u8, b: u8) -> pal::Color {
    pal::Color::rgb(r, g, b)
}