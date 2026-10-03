// ---------------------------------------------------------------------------
// engine.ts - WebGL2 frame pipeline. One draw call set per frame:
//   sky -> idol(1/2 res) -> merge -> bright(1/2) -> blur H -> blur V -> post
// ---------------------------------------------------------------------------

import { VERT, SKY_FRAG, GEO_FRAG, MERGE_FRAG, BRIGHT_FRAG, BLUR_FRAG, POST_FRAG } from "./shaders";
import { buildGlyphAtlas } from "./glyphs";
import type { GlState } from "../lib/timeline";

type Prog = { p: WebGLProgram; u: Record<string, WebGLUniformLocation | null> };

const compile = (gl: WebGL2RenderingContext, type: number, src: string, name: string): WebGLShader => {
  const s = gl.createShader(type)!;
  gl.shaderSource(s, src);
  gl.compileShader(s);
  if (!gl.getShaderParameter(s, gl.COMPILE_STATUS)) {
    throw new Error(`${name}: ${gl.getShaderInfoLog(s)}`);
  }
  return s;
};

const program = (gl: WebGL2RenderingContext, fs: string, name: string): Prog => {
  const p = gl.createProgram()!;
  gl.attachShader(p, compile(gl, gl.VERTEX_SHADER, VERT, name + ".vs"));
  gl.attachShader(p, compile(gl, gl.FRAGMENT_SHADER, fs, name + ".fs"));
  gl.linkProgram(p);
  if (!gl.getProgramParameter(p, gl.LINK_STATUS)) {
    throw new Error(`${name} link: ${gl.getProgramInfoLog(p)}`);
  }
  const u: Record<string, WebGLUniformLocation | null> = {};
  const n = gl.getProgramParameter(p, gl.ACTIVE_UNIFORMS) as number;
  for (let i = 0; i < n; i++) {
    const info = gl.getActiveUniform(p, i)!;
    const nm = info.name.replace(/\[0\]$/, "");
    u[nm] = gl.getUniformLocation(p, nm);
  }
  return { p, u };
};

type RT = { tex: WebGLTexture; fb: WebGLFramebuffer; w: number; h: number };

export class Engine {
  gl: WebGL2RenderingContext;
  private sky: Prog;
  private geo: Prog;
  private merge: Prog;
  private bright: Prog;
  private blur: Prog;
  private post: Prog;
  private rtScene: RT;
  private rtScene2: RT;
  private rtGeo: RT;
  private rtHalfA: RT;
  private rtHalfB: RT;
  private glyphTex: WebGLTexture;
  floatOK: boolean;

  constructor(canvas: HTMLCanvasElement) {
    const gl = canvas.getContext("webgl2", {
      antialias: false,
      alpha: false,
      depth: false,
      stencil: false,
      preserveDrawingBuffer: true,
      powerPreference: "high-performance",
    });
    if (!gl) throw new Error("WebGL2 unavailable");
    this.gl = gl;
    this.sky = program(gl, SKY_FRAG, "sky");
    this.geo = program(gl, GEO_FRAG, "geo");
    this.merge = program(gl, MERGE_FRAG, "merge");
    this.bright = program(gl, BRIGHT_FRAG, "bright");
    this.blur = program(gl, BLUR_FRAG, "blur");
    this.post = program(gl, POST_FRAG, "post");

    const W = canvas.width;
    const H = canvas.height;
    const hw = W >> 1;
    const hh = H >> 1;
    this.floatOK = !!gl.getExtension("EXT_color_buffer_float");
    const fmt = this.floatOK ? gl.RGBA16F : gl.RGBA8;
    const typ = this.floatOK ? gl.HALF_FLOAT : gl.UNSIGNED_BYTE;
    this.rtScene = this.makeRT(W, H, fmt, typ);
    this.rtScene2 = this.makeRT(W, H, fmt, typ);
    this.rtGeo = this.makeRT(W, H, fmt, typ);   // full res: geometry must stay crisp
    this.rtHalfA = this.makeRT(hw, hh, fmt, typ);
    this.rtHalfB = this.makeRT(hw, hh, fmt, typ);

    this.glyphTex = gl.createTexture()!;
    gl.bindTexture(gl.TEXTURE_2D, this.glyphTex);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, buildGlyphAtlas());
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
  }

  private makeRT(w: number, h: number, internal: number, type: number): RT {
    const gl = this.gl;
    const tex = gl.createTexture()!;
    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.texImage2D(gl.TEXTURE_2D, 0, internal, w, h, 0, gl.RGBA, type, null);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    const fb = gl.createFramebuffer()!;
    gl.bindFramebuffer(gl.FRAMEBUFFER, fb);
    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, tex, 0);
    gl.bindFramebuffer(gl.FRAMEBUFFER, null);
    return { tex, fb, w, h };
  }

  private bind(rt: RT | null) {
    const gl = this.gl;
    if (rt) {
      gl.bindFramebuffer(gl.FRAMEBUFFER, rt.fb);
      gl.viewport(0, 0, rt.w, rt.h);
    } else {
      gl.bindFramebuffer(gl.FRAMEBUFFER, null);
      gl.viewport(0, 0, this.rtScene.w, this.rtScene.h);
    }
  }

  private drawQuad() {
    this.gl.drawArrays(this.gl.TRIANGLES, 0, 3);
  }

  private use(prog: Prog, target: RT | null) {
    const gl = this.gl;
    gl.useProgram(prog.p);
    this.bind(target);
  }

  private tex(prog: Prog, name: string, texture: WebGLTexture, unit: number) {
    const gl = this.gl;
    gl.activeTexture(gl.TEXTURE0 + unit);
    gl.bindTexture(gl.TEXTURE_2D, texture);
    gl.uniform1i(prog.u[name]!, unit);
  }

  draw(s: GlState, frame: number) {
    const gl = this.gl;
    const W = this.rtScene.w;
    const H = this.rtScene.h;
    const hw = this.rtHalfA.w;
    const hh = this.rtHalfA.h;
    const p = s.pal;
    const t = frame / 60;
    const pan = [s.camX / 1080, -s.camY / 1080] as [number, number];
    const pulse = s.pulseVal;

    // 1. sky + base gradient into rtScene
    const sky = this.sky;
    this.use(sky, this.rtScene);
    gl.uniform2f(sky.u.uRes!, W, H);
    gl.uniform1f(sky.u.uT!, t);
    gl.uniform2f(sky.u.uCam!, s.camX / W, -s.camY / H);
    gl.uniform1f(sky.u.uZoom!, s.camZoom);
    gl.uniform3fv(sky.u.uC0!, p.c0);
    gl.uniform3fv(sky.u.uC1!, p.c1);
    gl.uniform3fv(sky.u.uC2!, p.c2);
    gl.uniform3fv(sky.u.uKey!, p.key);
    gl.uniform3fv(sky.u.uAlt!, p.alt);
    gl.uniform3fv(sky.u.uHot!, p.hot);
    gl.uniform1f(sky.u.uNeb!, s.neb);
    gl.uniform1f(sky.u.uNebScale!, s.nebScale);
    gl.uniform1f(sky.u.uFlow!, s.flow);
    gl.uniform1f(sky.u.uStars!, s.stars);
    gl.uniform1f(sky.u.uRays!, s.rays);
    gl.uniform1f(sky.u.uGrid!, s.grid);
    gl.uniform1f(sky.u.uGridSpeed!, s.gridSpeed);
    gl.uniform1f(sky.u.uPulse!, pulse);
    gl.uniform2fv(sky.u.uFocal!, s.focal);
    gl.uniform1f(sky.u.uFocalR!, s.focalR);
    gl.uniform1f(sky.u.uSeed!, s.seed);
    gl.uniform1f(sky.u.uGlitch!, s.glitch);
    gl.uniform1f(sky.u.uComet!, s.comet);
    gl.uniform1f(sky.u.uCometT0!, s.cometT0);
    gl.uniform1f(sky.u.uRingGain!, s.ringGain);
    gl.uniform2fv(sky.u.uPan!, pan);
    if (sky.u.uRings) gl.uniform2fv(sky.u.uRings, s.rings);
    this.drawQuad();

    // 2. analytic geometry at full res
    const id = this.geo;
    this.use(id, this.rtGeo);
    gl.clearColor(0, 0, 0, 0);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.uniform2f(id.u.uRes!, W, H);
    gl.uniform1f(id.u.uT!, t);
    gl.uniform1i(id.u.uGeo!, Math.round(s.geo));
    gl.uniform4f(id.u.uA!, s.geoA[0], s.geoA[1], s.geoA[2], s.geoA[3]);
    gl.uniform4f(id.u.uB!, s.geoB[0], s.geoB[1], s.geoB[2], s.geoB[3]);
    gl.uniform3fv(id.u.uKey!, p.key);
    gl.uniform3fv(id.u.uAlt!, p.alt);
    gl.uniform3fv(id.u.uHot!, p.hot);
    gl.uniform1f(id.u.uZoom!, s.camZoom);
    gl.uniform1f(id.u.uPulse!, pulse);
    gl.uniform2fv(id.u.uPan!, pan);
    this.drawQuad();

    // 3. merge sky+geometry+particles into rtScene2
    const mg = this.merge;
    this.use(mg, this.rtScene2);
    this.tex(mg, "uScene", this.rtScene.tex, 0);
    this.tex(mg, "uIdol", this.rtGeo.tex, 1);
    this.tex(mg, "uGlyph", this.glyphTex, 2);
    gl.uniform2f(mg.u.uRes!, W, H);
    gl.uniform1f(mg.u.uT!, t);
    gl.uniform1f(mg.u.uPulse!, pulse);
    gl.uniform1f(mg.u.uPartMode!, s.partMode);
    gl.uniform1f(mg.u.uPartAmt!, s.partAmt);
    gl.uniform3fv(mg.u.uPartCol!, s.partCol);
    gl.uniform3fv(mg.u.uPartCol2!, s.partCol2);
    gl.uniform1f(mg.u.uSeed!, s.seed);
    this.drawQuad();

    // 4. bright pass to halfA
    const br = this.bright;
    this.use(br, this.rtHalfA);
    this.tex(br, "uTex", this.rtScene2.tex, 0);
    gl.uniform2f(br.u.uRes!, hw, hh);
    gl.uniform1f(br.u.uThresh!, 0.82);
    this.drawQuad();

    // 5. blur H: halfA -> halfB, blur V: halfB -> halfA
    const bl = this.blur;
    this.use(bl, this.rtHalfB);
    this.tex(bl, "uTex", this.rtHalfA.tex, 0);
    gl.uniform2f(bl.u.uRes!, hw, hh);
    gl.uniform2f(bl.u.uDir!, 1.0 / hw, 0);
    this.drawQuad();
    this.use(bl, this.rtHalfA);
    this.tex(bl, "uTex", this.rtHalfB.tex, 0);
    gl.uniform2f(bl.u.uRes!, hw, hh);
    gl.uniform2f(bl.u.uDir!, 0, 1.0 / hh);
    this.drawQuad();

    // 6. post to screen
    const po = this.post;
    this.use(po, null);
    this.tex(po, "uScene", this.rtScene2.tex, 0);
    this.tex(po, "uBloom", this.rtHalfA.tex, 1);
    gl.uniform2f(po.u.uRes!, W, H);
    gl.uniform1f(po.u.uBloomAmt!, s.bloom);
    gl.uniform1f(po.u.uStreak!, s.streak);
    gl.uniform1f(po.u.uChroma!, s.chroma);
    gl.uniform1f(po.u.uVig!, s.vig);
    gl.uniform1f(po.u.uGrain!, s.grain);
    gl.uniform1f(po.u.uScan!, s.scan);
    gl.uniform1f(po.u.uSharp!, s.sharp);
    gl.uniform1f(po.u.uFlash!, s.flash);
    gl.uniform1f(po.u.uFade!, s.fade);
    gl.uniform1f(po.u.uExposure!, s.exposure);
    gl.uniform1f(po.u.uSat!, s.sat);
    gl.uniform1f(po.u.uFrame!, frame % 1024);
    gl.uniform3fv(po.u.uFlashCol!, s.flashCol);
    this.drawQuad();
  }
}
