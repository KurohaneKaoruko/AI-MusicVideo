// ---------------------------------------------------------------------------
// engine.js - render targets, pass orchestration, sprite batching
// ---------------------------------------------------------------------------

import { createGL, program, makeQuad, makeRT } from "./core.js";
import * as S from "./shaders.js";

const MAX_ADD = 7000;
const MAX_SOLID = 1800;

export class Engine {
  constructor(canvas) {
    this.canvas = canvas;
    this.W = canvas.width;
    this.H = canvas.height;
    const gl = (this.gl = createGL(canvas));

    this.quad = makeQuad(gl);
    this.pNebula = program(gl, S.VS_QUAD, S.FS_NEBULA, "nebula");
    this.pBg = program(gl, S.VS_QUAD, S.FS_BG, "bg");
    this.pSprite = program(gl, S.VS_SPRITE, S.FS_SPRITE, "sprite");
    this.pSolid = program(gl, S.VS_SPRITE, S.FS_SPRITE_SOLID, "solid");
    this.pBright = program(gl, S.VS_QUAD, S.FS_BRIGHT, "bright");
    this.pTex = program(gl, S.VS_QUAD, S.FS_TEX, "tex");
    this.pBlur = program(gl, S.VS_QUAD, S.FS_BLUR, "blur");
    this.pUp = program(gl, S.VS_QUAD, S.FS_UP, "up");
    this.pPost = program(gl, S.VS_QUAD, S.FS_POST, "post");

    const floatOK = !!gl.getExtension("EXT_color_buffer_float") ||
                    !!gl.getExtension("EXT_color_buffer_half_float");
    this.floatOK = floatOK;
    const F = { float: floatOK };

    // nebula is deliberately tiny (1/8 linear -> 1/64 of the pixels at 960x540?)
    this.nebW = 480; this.nebH = 270;
    this.rtNeb = makeRT(gl, this.nebW, this.nebH, { float: false, linear: true });
    this.rtScene = makeRT(gl, this.W, this.H, F);
    this.rtText = makeRT(gl, this.W, this.H, { float: false, linear: true });
    // the graded image lands here, never on the default framebuffer: reading
    // back the canvas means going through Chrome's compositor, which is both
    // slow and non-deterministic. An FBO read is a straight GPU->CPU copy.
    this.rtOut = makeRT(gl, this.W, this.H, { float: false, linear: false });

    // bloom pyramid
    const lv = [[this.W >> 1, this.H >> 1], [this.W >> 2, this.H >> 2], [this.W >> 3, this.H >> 3]];
    this.bloom = lv.map(([w, h]) => ({
      a: makeRT(gl, Math.max(2, w), Math.max(2, h), F),
      b: makeRT(gl, Math.max(2, w), Math.max(2, h), F),
      w: Math.max(2, w), h: Math.max(2, h),
    }));

    // ------------------------------------------------ sprite data texture ---
    this.addData = new Float32Array(MAX_ADD * 12);
    this.addCount = 0;
    this.solidData = new Float32Array(MAX_SOLID * 12);
    this.solidCount = 0;
    this.texAdd = this._dataTex(MAX_ADD);
    this.texSolid = this._dataTex(MAX_SOLID);

    // ------------------------------------------------------- text overlay ---
    this.textCanvas = document.createElement("canvas");
    this.textCanvas.width = this.W;
    this.textCanvas.height = this.H;
    this.ctx = this.textCanvas.getContext("2d", { alpha: true });
    // Skia charges an absurd amount for shadowBlur on a 1080p `lighter` canvas
    // (measured: ~1.5 s per frame with the recursive tree + silhouettes). The
    // film's glow already comes from the GL bloom pyramid, which is paid for
    // whether or not anything glows, so the 2D shadow is pure waste: make it a
    // no-op and let bloom do the halo.
    try {
      Object.defineProperty(this.ctx, "shadowBlur", {
        get: () => 0, set: () => {}, configurable: true,
      });
    } catch { /* non-configurable on some builds - fall back to the flag */ }
    this.texText = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, this.texText);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, this.W, this.H, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    this.textDirty = true;
    // "tex" (realloc every frame) | "sub" (preallocated) | "getdata" | "off"
    this.textUploadMode = "tex";

    // when captureFlip is on, the final post pass is skipped on screen and the
    // rtOut framebuffer is read back directly. flipY additionally mirrors the
    // pass; on this ANGLE/D3D11 backend readPixels already hands rows back in
    // image order, so flipY stays 0 (measured, not assumed).
    this.captureFlip = false;
    this.flipY = false;
    this.pxBuf = new Uint8Array(this.W * this.H * 4);

    this.blit = this.blit.bind(this);
  }

  _dataTex(n) {
    const gl = this.gl;
    const t = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, t);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA32F, 12, n, 0, gl.RGBA, gl.FLOAT, null);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    return t;
  }

  // ------------------------------------------------------------ primitives ---

  push(count, limit, buf, x, y, size, rot, col, alpha, shape, p1, p2, p3) {
    if (count >= limit) return;
    const o = count * 12;
    buf[o] = x; buf[o + 1] = y; buf[o + 2] = size; buf[o + 3] = rot;
    buf[o + 4] = col[0]; buf[o + 5] = col[1]; buf[o + 6] = col[2]; buf[o + 7] = alpha;
    buf[o + 8] = shape; buf[o + 9] = p1; buf[o + 10] = p2; buf[o + 11] = p3;
  }

  sprite(x, y, size, rot, col, alpha, shape = 0, p1 = 0.5, p2 = 0.1, p3 = 0.1) {
    this.push(this.addCount, MAX_ADD, this.addData, x, y, size, rot, col, alpha, shape, p1, p2, p3);
    if (this.addCount < MAX_ADD) this.addCount++;
  }

  solid(x, y, size, rot, col, alpha, shape = 6, p1 = 0.5, p2 = 0.5, p3 = 0.05) {
    this.push(this.solidCount, MAX_SOLID, this.solidData, x, y, size, rot, col, alpha, shape, p1, p2, p3);
    if (this.solidCount < MAX_SOLID) this.solidCount++;
  }

  // ----------------------------------------------------------------- passes ---

  blit(prog, target, setUniforms) {
    const gl = this.gl;
    if (target) target.bind();
    else { gl.bindFramebuffer(gl.FRAMEBUFFER, null); gl.viewport(0, 0, this.W, this.H); }
    gl.useProgram(prog.prog);
    if (setUniforms) setUniforms(prog.u, gl);
    gl.bindVertexArray(this.quad);
    gl.disable(gl.BLEND);
    gl.drawArrays(gl.TRIANGLES, 0, 3);
  }

  bindTex(loc, unit, tex) {
    const gl = this.gl;
    gl.activeTexture(gl.TEXTURE0 + unit);
    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.uniform1i(loc, unit);
  }

  /** run a full frame. state comes from the director. */
  render(state, frame) {
    const gl = this.gl;
    const st = state;

    // ---- 1. nebula at 1/16 resolution -------------------------------------
    this.rtNeb.bind();
    gl.useProgram(this.pNebula.prog);
    {
      const u = this.pNebula.u;
      gl.uniform1f(u.uT, st.bg.t);
      gl.uniform1f(u.uScale, st.bg.scale);
      gl.uniform1f(u.uWarp, st.bg.warp);
      gl.uniform1f(u.uFlow, st.bg.flow);
      gl.uniform1f(u.uContrast, st.bg.contrast);
      gl.uniform1f(u.uGain, st.bg.gain);
      gl.uniform1f(u.uAspect, st.aspect);
      gl.uniform2f(u.uOff, st.bg.offx, st.bg.offy);
      gl.uniform3fv(u.uC0, st.bg.c0);
      gl.uniform3fv(u.uC1, st.bg.c1);
      gl.uniform3fv(u.uC2, st.bg.c2);
    }
    gl.bindVertexArray(this.quad);
    gl.disable(gl.BLEND);
    gl.drawArrays(gl.TRIANGLES, 0, 3);

    // ---- 2. background composite -----------------------------------------
    this.rtScene.bind();
    gl.useProgram(this.pBg.prog);
    {
      const u = this.pBg.u;
      this.bindTex(u.uNeb, 0, this.rtNeb.tex);
      gl.uniform1f(u.uT, st.bg.t);
      gl.uniform1f(u.uAspect, st.aspect);
      gl.uniform1f(u.uZoom, st.cam.zoom);
      gl.uniform1f(u.uRot, st.cam.rot);
      gl.uniform2f(u.uPanX, st.cam.x, st.cam.y);
      gl.uniform3fv(u.uC0, st.bg.c0);
      gl.uniform3fv(u.uC1, st.bg.c1);
      gl.uniform3fv(u.uC2, st.bg.c2);
      gl.uniform3fv(u.uGridCol, st.bg.gridCol);
      gl.uniform3fv(u.uShaftCol, st.bg.shaftCol);
      gl.uniform3fv(u.uGlowCol, st.bg.glowCol);
      gl.uniform1f(u.uGridAmt, st.bg.gridAmt);
      gl.uniform1f(u.uGridMode, st.bg.gridMode);
      gl.uniform1f(u.uGridScale, st.bg.gridScale);
      gl.uniform1f(u.uGridDepth, st.bg.gridDepth);
      gl.uniform1f(u.uShaftAmt, st.bg.shaftAmt);
      gl.uniform1f(u.uShaftAng, st.bg.shaftAng);
      gl.uniform1f(u.uShaftWide, st.bg.shaftWide);
      gl.uniform1f(u.uGlowAmt, st.bg.glowAmt);
      gl.uniform1f(u.uGlowR, st.bg.glowR);
      gl.uniform1f(u.uGlowX, st.bg.glowX);
      gl.uniform1f(u.uGlowY, st.bg.glowY);
      gl.uniform1f(u.uNebMix, st.bg.nebMix);
      gl.uniform1f(u.uHaze, st.bg.haze);
      gl.uniform1f(u.uFloor, st.bg.floor);
    }
    gl.bindVertexArray(this.quad);
    gl.drawArrays(gl.TRIANGLES, 0, 3);

    // ---- 3. sprites (additive) -------------------------------------------
    if (this.addCount > 0) {
      this.uploadSprites(this.texAdd, this.addData, this.addCount);
      this.drawSprites(this.pSprite, this.texAdd, this.addCount, st, gl.ONE, gl.ONE, true);
    }

    // ---- 4. text layer (additive, before bloom so it glows) ---------------
    if (st.textActive) {
      this.uploadText();
      gl.enable(gl.BLEND);
      gl.blendFunc(gl.ONE, gl.ONE);
      this.rtScene.bind();
      gl.useProgram(this.pTex.prog);
      {
        const u = this.pTex.u;
        this.bindTex(u.uTex, 0, this.texText);
        gl.uniform1f(u.uGain, st.textGain);
        gl.uniform1f(u.uAlpha, 1.0);
        gl.uniform3fv(u.uMul, st.textTint);
      }
      gl.bindVertexArray(this.quad);
      gl.drawArrays(gl.TRIANGLES, 0, 3);
      gl.disable(gl.BLEND);
    }

    // ---- 5. solids (alpha blended over everything) ------------------------
    if (this.solidCount > 0) {
      this.uploadSprites(this.texSolid, this.solidData, this.solidCount);
      this.drawSprites(this.pSolid, this.texSolid, this.solidCount, st,
                       gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA, false);
    }

    // ---- 6. bloom ----------------------------------------------------------
    const bl = this.bloom;
    const thresh = st.fx.bloomThresh;
    this.blit(this.pBright, bl[0].a, (u) => {
      this.bindTex(u.uTex, 0, this.rtScene.tex);
      gl.uniform1f(u.uThresh, thresh);
      gl.uniform1f(u.uKnee, 0.55);
      gl.uniform1f(u.uIntensity, 1.0);
    });
    for (let i = 0; i < bl.length; i++) {
      if (i > 0) {
        this.blit(this.pBlur, bl[i].a, (u) => {
          this.bindTex(u.uTex, 0, bl[i - 1].a.tex);
          gl.uniform2f(u.uDir, 1 / bl[i].w, 0);
        });
      }
      this.blit(this.pBlur, bl[i].b, (u) => {
        this.bindTex(u.uTex, 0, bl[i].a.tex);
        gl.uniform2f(u.uDir, 0, 1 / bl[i].h);
      });
      this.blit(this.pBlur, bl[i].a, (u) => {
        this.bindTex(u.uTex, 0, bl[i].b.tex);
        gl.uniform2f(u.uDir, 1 / bl[i].w, 0);
      });
    }
    for (let i = bl.length - 2; i >= 0; i--) {
      this.blit(this.pUp, bl[i].a, (u) => {
        this.bindTex(u.uTex, 0, bl[i].a.tex);
        this.bindTex(u.uAdd, 1, bl[i + 1].a.tex);
        gl.uniform1f(u.uMix, 0.72);
      });
    }

    // ---- 7. final post ------------------------------------------------------
    this.rtOut.bind();
    gl.useProgram(this.pPost.prog);
    {
      const u = this.pPost.u;
      this.bindTex(u.uScene, 0, this.rtScene.tex);
      this.bindTex(u.uBloom, 1, bl[0].a.tex);
      gl.uniform2f(u.uRes, this.W, this.H);
      gl.uniform1f(u.uT, st.t);
      gl.uniform1f(u.uFrame, frame);
      gl.uniform1f(u.uBloomAmt, st.fx.bloom);
      gl.uniform1f(u.uChroma, st.fx.chroma);
      gl.uniform1f(u.uChromaR, st.fx.chromaR);
      gl.uniform1f(u.uGlitch, st.fx.glitch);
      gl.uniform1f(u.uGlitchSeed, st.fx.glitchSeed);
      gl.uniform1f(u.uRows, st.fx.rows);
      gl.uniform1f(u.uScan, st.fx.scan);
      gl.uniform1f(u.uGrain, st.fx.grain);
      gl.uniform1f(u.uVignette, st.fx.vignette);
      gl.uniform1f(u.uFlash, st.fx.flash);
      gl.uniform1f(u.uFade, st.fx.fade);
      gl.uniform1f(u.uSat, st.fx.sat);
      gl.uniform1f(u.uContrast, st.fx.contrast);
      gl.uniform1f(u.uLift, st.fx.lift);
      gl.uniform1f(u.uSmear, st.fx.smear);
      gl.uniform1f(u.uSmearY, st.fx.smearY);
      gl.uniform1f(u.uExposure, st.fx.exposure);
      gl.uniform3fv(u.uFlashCol, st.fx.flashCol);
      gl.uniform3fv(u.uTint, st.fx.tint);
      gl.uniform3fv(u.uLiftCol, st.fx.liftCol);
      gl.uniform1f(u.uFlipY, this.flipY ? 1.0 : 0.0);
    }
    gl.bindVertexArray(this.quad);
    gl.disable(gl.BLEND);
    gl.drawArrays(gl.TRIANGLES, 0, 3);

    // ---- 8. present (skipped entirely while capturing) ----------------------
    if (!this.captureFlip) {
      this.blit(this.pTex, null, (u) => {
        this.bindTex(u.uTex, 0, this.rtOut.tex);
        gl.uniform1f(u.uGain, 1.0);
        gl.uniform1f(u.uAlpha, 1.0);
        gl.uniform3f(u.uMul, 1, 1, 1);
      });
    }
  }

  /** raw RGBA dump of the graded frame, top-down after captureFlip */
  readback() {
    const gl = this.gl;
    this.rtOut.bind();
    gl.readPixels(0, 0, this.W, this.H, gl.RGBA, gl.UNSIGNED_BYTE, this.pxBuf);
    return this.pxBuf;
  }

  uploadSprites(tex, data, count) {
    const gl = this.gl;
    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, 12, count, gl.RGBA, gl.FLOAT,
                     data.subarray(0, count * 12));
  }

  drawSprites(p, tex, count, st, src, dst, additive) {
    const gl = this.gl;
    this.rtScene.bind();
    gl.useProgram(p.prog);
    this.bindTex(p.u.uData, 0, tex);
    gl.uniform2f(p.u.uRes, this.W, this.H);
    gl.uniform2f(p.u.uCam, st.cam.x, st.cam.y);
    gl.uniform1f(p.u.uZoom, st.cam.zoom);
    gl.uniform1f(p.u.uRot, st.cam.rot);
    gl.uniform1f(p.u.uAA, 0.012);
    gl.uniform1i(p.u.uCount, count);
    gl.enable(gl.BLEND);
    gl.blendFunc(src, dst);
    gl.bindVertexArray(this.quad);
    gl.drawArraysInstanced(gl.TRIANGLE_STRIP, 0, 4, count);
    gl.disable(gl.BLEND);
  }

  uploadText() {
    const gl = this.gl;
    const m = this.textUploadMode;
    if (m === "off") { this.textDirty = false; return; }
    gl.bindTexture(gl.TEXTURE_2D, this.texText);
    if (m === "sub") {
      // storage was already allocated in the constructor - no realloc per frame
      gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, this.W, this.H, gl.RGBA, gl.UNSIGNED_BYTE,
                       this.textCanvas);
    } else if (m === "getdata") {
      const d = this.ctx.getImageData(0, 0, this.W, this.H);
      gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, this.W, this.H, gl.RGBA, gl.UNSIGNED_BYTE, d.data);
    } else {
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, this.W, this.H, 0, gl.RGBA, gl.UNSIGNED_BYTE,
                    this.textCanvas);
    }
    this.textDirty = false;
  }
}
