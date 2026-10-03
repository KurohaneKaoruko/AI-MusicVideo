// ---------------------------------------------------------------------------
// shaders.ts - GLSL sources for the v3 pipeline.
//   sky   : volumetric nebula + starfield + god rays + perspective floor
//   idol  : raymarched SDF centrepiece (morphing machine-idol)
//   merge : composite sky + idol + procedural particle fields
//   bright/blur : bloom chain
//   post  : anamorphic streak + chroma + grain + scanline + vignette + ACES
// ---------------------------------------------------------------------------

export const VERT = `#version 300 es
void main(){
  vec2 p = vec2(float((gl_VertexID << 1) & 2), float(gl_VertexID & 2));
  gl_Position = vec4(p * 2.0 - 1.0, 0.0, 1.0);
}`;

const COMMON = `
float n21(vec2 p){
  vec3 p3 = fract(vec3(p.xyx) * 0.1031);
  p3 += dot(p3, p3.yzx + 33.33);
  return fract((p3.x + p3.y) * p3.z);
}
float noise2(vec2 p){
  vec2 i = floor(p), f = fract(p);
  f = f * f * (3.0 - 2.0 * f);
  return mix(mix(n21(i), n21(i + vec2(1, 0)), f.x),
             mix(n21(i + vec2(0, 1)), n21(i + vec2(1, 1)), f.x), f.y);
}
float fbm(vec2 p){
  float s = 0.0, a = 0.5;
  mat2 r = mat2(0.8, 0.6, -0.6, 0.8);
  for (int i = 0; i < 4; i++){ s += a * noise2(p); p = r * p * 2.02; a *= 0.5; }
  return s;
}
`;

// ------------------------------------------------------------------- sky ----

export const SKY_FRAG = `#version 300 es
precision highp float;
out vec4 O;
uniform vec2 uRes;
uniform float uT;
uniform vec2 uCam;
uniform float uZoom;
uniform vec3 uC0, uC1, uC2, uKey, uAlt, uHot;
uniform float uNeb, uNebScale, uFlow, uStars, uRays, uGrid, uGridSpeed, uPulse;
uniform vec2 uFocal;
uniform float uFocalR;
uniform float uSeed;
uniform float uGlitch;
uniform float uComet;
uniform float uCometT0;
uniform float uRingGain;
uniform vec2 uPan;
uniform vec2 uRings[6];
${COMMON}
vec2 cometPos(float u){
  float e = 1.0 - pow(1.0 - clamp(u, 0.0, 1.0), 3.0);
  float x = mix(-0.78, 0.42, e);
  float y = 0.02 + 0.30 * sin(clamp(u, 0.0, 1.0) * 2.67) - 0.02 * u + 0.012 * sin(u * 21.0);
  return vec2(x, y);
}
void main(){
  vec2 uv = gl_FragCoord.xy / uRes;
  vec2 p = uv - 0.5;
  p.x *= uRes.x / uRes.y;
  if (uGlitch > 0.002){
    float sl = floor(uv.y * 30.0);
    float on = step(0.74, n21(vec2(sl, floor(uT * 17.0) + uSeed)));
    p.x += (n21(vec2(sl * 7.3, floor(uT * 17.0))) - 0.5) * 0.13 * uGlitch * on;
  }
  vec2 sp = (p - uPan) / uZoom;
  vec2 screenP = p * vec2(uRes.x / uRes.y, 1.0);   // screen-anchored space

  float g = clamp(sp.y * 0.75 + 0.5, 0.0, 1.0);
  vec3 col = mix(uC0, uC1, pow(g, 1.5));

  // nebula: domain-warped fbm
  vec2 np = sp * uNebScale + vec2(uT * 0.012 * uFlow, -uT * 0.008 * uFlow) + uSeed;
  vec2 q = vec2(fbm(np), fbm(np + vec2(4.7, 2.3) + uSeed));
  float f = fbm(np + 2.3 * q);
  float f2 = fbm(np * 2.13 - q * 1.7 + uT * 0.02 * uFlow);
  vec3 neb = mix(uC0 * 2.0, uC1, smoothstep(0.2, 0.9, f));
  neb = mix(neb, uC2 * 1.25, pow(smoothstep(0.42, 1.05, f * f2 * 1.6), 1.35));
  col += neb * uNeb * (0.7 + 0.55 * f2);

  // starfield: two parallax layers
  float st = 0.0;
  {
    vec2 s1 = sp * 22.0;
    vec2 id = floor(s1); vec2 fr = fract(s1) - 0.5;
    float h = n21(id);
    vec2 o = vec2(n21(id + 7.1), n21(id + 3.7)) - 0.5;
    float d = length(fr - o * 0.8);
    float tw = 0.55 + 0.45 * sin(uT * (1.0 + h * 4.0) + h * 40.0);
    st += exp(-d * d * (700.0 - h * 420.0)) * step(0.80, h) * tw;
    vec2 s2 = sp * 47.0 + 31.7;
    id = floor(s2); fr = fract(s2) - 0.5;
    h = n21(id + 13.1);
    o = vec2(n21(id + 3.3), n21(id + 9.9)) - 0.5;
    d = length(fr - o * 0.8);
    st += exp(-d * d * 1100.0) * step(0.86, h) * (0.5 + 0.5 * sin(uT * (2.0 + h * 5.0) + h * 30.0)) * 0.7;
  }
  col += uHot * st * uStars;

  // god rays from focal point (uFocal lives in world space, like sp)
  vec2 fp = uFocal;
  fp.x *= uRes.x / uRes.y;
  vec2 dir = sp - fp;
  float r = length(dir);
  float an = atan(dir.y, dir.x);
  float rays = fbm(vec2(an * 3.0 + uT * 0.05 * uFlow, uSeed)) * fbm(vec2(an * 7.0 - uT * 0.03, uSeed + 9.0));
  rays = pow(max(rays, 0.0), 2.1);
  float fall = exp(-r * 2.1 / max(uFocalR, 0.05));
  col += mix(uC2, uKey, 0.5) * rays * fall * uRays * 1.5;
  col += uKey * exp(-r * r / max(uFocalR * uFocalR * 0.16, 1e-5)) * (0.20 + 0.45 * uRays) * (0.85 + 0.3 * uPulse);

  // beat shockwave rings
  for (int i = 0; i < 6; i++){
    float age = uRings[i].x;
    float str = uRings[i].y;
    if (age < 2.0 && str > 0.01){
      float rr = age * 0.9;
      float ring = exp(-pow((r - rr) / 0.016, 2.0));
      float fade = exp(-age * 3.4) * str;
      col += uKey * ring * fade * uRingGain;
    }
  }

  // the blue fairy: a comet with a long fading trail (screen-anchored)
  if (uComet > 0.001){
    float u = (uT - uCometT0) / 2.6;
    vec3 ct = vec3(0.0);
    for (int i = 0; i < 34; i++){
      float back = float(i) * 0.0085;
      float uu = u - back;
      float ok = step(0.0, uu) * step(uu, 1.0);
      vec2 cp = cometPos(min(uu, 1.0));
      cp.x *= uRes.x / uRes.y;
      float d = length(screenP - cp);
      float k = exp(-back * 6.0) * ok;
      ct += uHot * exp(-d * d * (9000.0 * k + 2600.0)) * k;
      ct += uKey * exp(-d * d * 2000.0) * k * 0.55;
    }
    col += ct * uComet;
  }

  // perspective floor
  if (uGrid > 0.001 && sp.y < 0.16){
    float z = 1.0 / ((0.16 - sp.y) * 1.4 + 0.09);
    vec2 gp = vec2(sp.x * z, z + uT * uGridSpeed);
    vec2 fw = fwidth(gp) * 1.3;
    vec2 fl = abs(fract(gp) - 0.5);
    vec2 ln = 1.0 - smoothstep(vec2(0.0), fw * 2.0 + 0.035, fl - 0.47);
    float line = max(ln.x, ln.y);
    float fadeH = smoothstep(0.16, -0.2, sp.y);
    float distf = exp(-z * 0.16);
    col += uKey * line * fadeH * distf * uGrid * 0.55;
    col += uC2 * fadeH * distf * uGrid * 0.12;
  }

  O = vec4(col, 1.0);
}`;

// ------------------------------------------------------------------ geo ----
// Pure analytic geometry — the abstract figure of each movement. Crisp 2D
// shapes: vital line, discs, mandala, strings, cracks, orbs, slashes.

export const GEO_FRAG = `#version 300 es
precision highp float;
out vec4 O;
uniform vec2 uRes;
uniform float uT;
uniform int uGeo;         // 0 none 1 vital 2 disc 3 mandala 4 strings 5 cracks 6 orb 7 slashes 8 strike 9 doll
uniform vec4 uA;          // per-mode params: x cx, y cy, z R, w aux
uniform vec4 uB;          // per-mode params: x progress/speed, y aux, z aux, w amount
uniform vec3 uKey, uAlt, uHot;
uniform vec2 uPan;
uniform float uZoom;
uniform float uPulse;
#define TAU 6.28318530718
float n21(vec2 p){
  vec3 p3 = fract(vec3(p.xyx) * 0.1031);
  p3 += dot(p3, p3.yzx + 33.33);
  return fract((p3.x + p3.y) * p3.z);
}
void main(){
  vec2 uv = gl_FragCoord.xy / uRes;
  vec2 p = uv - 0.5;
  p.x *= uRes.x / uRes.y;
  float asp = uRes.x / uRes.y;
  vec2 w = (p - uPan) / uZoom;
  vec3 col = vec3(0.0);
  float a = 0.0;
  vec2 c = vec2(uA.x * asp, uA.y);
  float R = uA.z * (1.0 + 0.03 * uPulse);

  if (uGeo == 1){
    // vital line: beats spike, then flatline
    float ph = uv.x * 7.0 - uT * 0.9;
    float cell = floor(ph); float fr = fract(ph);
    float on = step(0.86, n21(vec2(cell, 7.0)));
    float sp = on * (exp(-pow((fr - 0.32) / 0.035, 2.0)) * 0.9
             - exp(-pow((fr - 0.38) / 0.03, 2.0)) * 0.4
             + exp(-pow((fr - 0.50) / 0.02, 2.0)) * 1.7);
    sp *= uB.z;
    float d = abs(uv.y - (uA.y + sp * 0.06));
    float m = smoothstep(0.0, 0.08, uv.x) * (1.0 - smoothstep(0.92, 1.0, uv.x));
    col += uKey * (exp(-d * d * 26000.0) * 0.95 + exp(-d * d * 2200.0) * 0.20) * m * uB.w;
    a = exp(-d * d * 26000.0) * m * uB.w;
  } else if (uGeo == 2){
    // disc: rim + inner ring + optional fill
    float d = length(w - c) - R;
    float ring = 1.0 - smoothstep(0.0, 0.0022, abs(d));
    float ring2 = 1.0 - smoothstep(0.0, 0.0012, abs(length(w - c) - R * 0.86));
    float fill = (1.0 - smoothstep(0.0, 0.004, -d)) * uA.w;
    vec3 rim = mix(uKey, uHot, 0.4) * (0.85 + 0.35 * uPulse);
    col = rim * ring * uB.w + uKey * ring2 * 0.5 * uB.w + uAlt * fill * 0.35;
    a = max(ring * uB.w, fill);
  } else if (uGeo == 3){
    // mandala: 12 petals orbiting a core
    vec2 q = w - c;
    float r = length(q);
    vec3 acc = vec3(0.0); float am = 0.0;
    for (int i = 0; i < 12; i++){
      float fi = float(i);
      vec2 dir = vec2(cos(fi * TAU / 12.0), sin(fi * TAU / 12.0));
      vec2 pp = q - dir * R;
      float rot = uT * uB.x + fi;
      pp = mat2(cos(rot), -sin(rot), sin(rot), cos(rot)) * pp;
      float d = length(pp * vec2(1.0, 0.42)) - R * 0.20;
      float pet = exp(-max(d, 0.0) * 150.0);
      acc += mix(uKey, uAlt, fi / 12.0) * pet * 0.18;
      am = max(am, pet * 0.55);
    }
    float core = exp(-pow(r / (R * 0.12), 2.0));
    acc += uHot * core * 0.8; am = max(am, core);
    float ring = 1.0 - smoothstep(0.0, 0.0018, abs(r - R * 1.22));
    acc += uKey * ring * 0.55; am = max(am, ring);
    col = acc * uB.w; a = am * uB.w;
  } else if (uGeo == 4 || uGeo == 9){
    // strings: five swaying verticals
    float amt = 0.0;
    for (int i = 0; i < 5; i++){
      float fi = float(i) - 2.0;
      float x0 = uA.x + fi * uA.z;
      float sway = sin(uT * (0.5 + abs(fi) * 0.13) + fi * 2.1) * 0.008;
      float d = abs(w.x - (x0 * asp + sway * (1.2 + w.y)));
      float line = exp(-d * d * 260000.0);
      float vfade = smoothstep(-0.55, -0.12, w.y) * (1.0 - smoothstep(0.55, 0.62, w.y));
      col += uKey * line * vfade * uB.w * 0.5;
      amt = max(amt, line * vfade);
    }
    a = max(a, amt * uB.w);
    if (uGeo == 9){
      // + radial cracks growing from center
      vec2 q = w - c;
      float r = length(q);
      float an = atan(q.y, q.x);
      for (int i = 0; i < 10; i++){
        float fi = float(i);
        float base = fi * TAU / 10.0 + n21(vec2(fi, 3.1)) * 0.5;
        float jit = (n21(vec2(floor(r * 24.0), fi * 17.0)) - 0.5) * 0.22;
        float d = abs(sin(an - base - jit)) * r;
        float ray = exp(-d * d * 90000.0)
                  * smoothstep(R * uB.x * 1.15, R * uB.x * 0.15, r)
                  * step(r, R * uB.x * 1.05);
        col += mix(uAlt, uHot, 0.35 + 0.5 * n21(vec2(fi, 9.7))) * ray * uB.w * 1.2;
        a = max(a, ray * uB.w);
      }
    }
  } else if (uGeo == 5){
    // cracks alone
    vec2 q = w - c;
    float r = length(q);
    float an = atan(q.y, q.x);
    for (int i = 0; i < 10; i++){
      float fi = float(i);
      float base = fi * TAU / 10.0 + n21(vec2(fi, 3.1)) * 0.5;
      float jit = (n21(vec2(floor(r * 24.0), fi * 17.0)) - 0.5) * 0.22;
      float d = abs(sin(an - base - jit)) * r;
      float ray = exp(-d * d * 90000.0)
                * smoothstep(R * uB.x * 1.15, R * uB.x * 0.15, r)
                * step(r, R * uB.x * 1.05);
      col += mix(uAlt, uHot, 0.35 + 0.5 * n21(vec2(fi, 9.7))) * ray * uB.w * 1.3;
      a = max(a, ray * uB.w);
    }
  } else if (uGeo == 6){
    // orb: core + halo + thin ring
    float d = length(w - c);
    float core = exp(-pow(d / max(R * 0.16, 1e-4), 2.0));
    float halo = exp(-pow(d / max(R * 0.6, 1e-4), 2.0)) * 0.4;
    float ring = 1.0 - smoothstep(0.0, 0.0016, abs(d - R));
    col = (uHot * core * 1.6 + uKey * halo + uKey * ring * (0.5 + 0.3 * uPulse)) * uB.w * (0.85 + 0.3 * uPulse);
    a = clamp(core + ring * 0.8, 0.0, 1.0) * uB.w;
  } else if (uGeo == 7){
    // slashes: three comets converge to center
    float k = clamp((uT - uA.w) / max(uB.x, 0.01), 0.0, 1.0);
    if (uB.y > 0.5 && k > 0.0){
      for (int i = 0; i < 3; i++){
        float fi = float(i);
        float ph = fi * TAU / 3.0;
        for (int j = 0; j < 14; j++){
          float back = float(j) * 0.012;
          float kk = max(k - back, 0.0);
          float ang = ph * (1.0 - kk * kk) + (fi - 1.0) * 0.12;
          float rr = mix(0.95, 0.03, 1.0 - pow(1.0 - kk, 2.0));
          vec2 cp = vec2(cos(ang), sin(ang)) * rr;
          cp.x *= asp;
          float d = length(p - cp);
          float f = exp(-back * 5.0);
          col += (uHot * exp(-d * d * (9000.0 * f + 2200.0)) * f * 1.1
                + uKey * exp(-d * d * 1800.0) * f * 0.45);
        }
      }
      a = clamp(dot(col, vec3(0.45)), 0.0, 1.0);
    }
  } else if (uGeo == 8){
    // strike: a line drawing across
    float xx = mix(uA.x, uA.z, clamp(uB.x, 0.0, 1.0)) * asp;
    float onX = step(uA.x * asp - 0.01, p.x) * step(p.x, xx);
    float d = abs(p.y - uA.y);
    float line = exp(-d * d * 90000.0) * onX;
    col = uAlt * line * 1.7 * uB.w;
    a = line * uB.w;
  }
  O = vec4(col, clamp(a, 0.0, 1.0));
}`;

// ---------------------------------------------------------- merge/particles --

export const MERGE_FRAG = `#version 300 es
precision highp float;
out vec4 O;
uniform vec2 uRes;
uniform sampler2D uScene;
uniform sampler2D uIdol;
uniform sampler2D uGlyph;
uniform float uT;
uniform float uPulse;
uniform float uPartMode;
uniform float uPartAmt;
uniform vec3 uPartCol;
uniform vec3 uPartCol2;
uniform float uSeed;
${COMMON}
#define TAU 6.28318530718

vec3 rainGlyphs(vec2 uv, float amt, vec3 col, vec3 head, float speed, float seed, float cols){
  vec2 g = vec2(uv.x * cols, (1.0 - uv.y) * cols * 0.62);
  float colId = floor(g.x);
  float cs = n21(vec2(colId, seed));
  float spd = speed * (0.5 + 1.2 * cs);
  float rows = cols * 0.62;
  float headRow = mod(cs * 97.0 + uT * spd, rows + 12.0) - 6.0;
  float row = floor(g.y);
  float x = headRow - row;
  float vis = step(-0.9, x) * step(x, 10.0);
  float trail = exp(-max(x, 0.0) * 0.40);
  float isHead = smoothstep(0.7, 0.0, abs(x - 0.25));
  float h1 = n21(vec2(colId * 3.1, row * 7.7 + floor(uT * 2.0) + seed));
  float h2 = n21(vec2(colId * 5.3, row * 2.9 + seed * 3.1));
  vec2 cell = vec2(floor(h1 * 15.99), floor(h2 * 15.99));
  vec2 auv = (cell + fract(vec2(g.x, g.y))) / 16.0;
  float glyph = texture(uGlyph, auv).a;
  float dens = step(0.25, cs);
  float a = glyph * vis * trail * dens * amt;
  return mix(col, head, isHead * 0.85) * a * (0.35 + 0.65 * cs);
}

vec3 flutter(vec2 uv, float amt, vec3 col, vec3 col2, float speed, float seed, float mode){
  vec3 acc = vec3(0.0);
  for (int L = 0; L < 2; L++){
    float fl = float(L);
    float density = 13.0 + fl * 9.0;
    vec2 g = vec2(uv.x * density, (1.0 - uv.y) * density * 0.8);
    g.y -= uT * speed * (0.6 + 0.5 * fl) * (mode == 3.0 ? -1.0 : 1.0);
    g.x += sin(uT * 0.6 + fl * 9.0 + g.y * 0.14) * (mode == 4.0 ? 0.7 : 1.3);
    vec2 id = floor(g);
    vec2 fr = fract(g) - 0.5;
    float h = n21(id + seed + fl * 17.0);
    vec2 o = (vec2(n21(id + 1.3 + fl), n21(id + 7.7 + fl)) - 0.5) * 0.7;
    vec2 q = fr - o;
    float rot = h * TAU + uT * (0.4 + h * 0.5);
    q = mat2(cos(rot), -sin(rot), sin(rot), cos(rot)) * q;
    float d;
    if (mode == 2.0)      d = length(q * vec2(1.0, 0.42)) - (0.065 + 0.05 * h);
    else if (mode == 4.0) d = length(q) - (0.035 + 0.03 * h);
    else                  d = length(q) - (0.018 + 0.022 * h);
    float flick = mode == 3.0 ? (0.55 + 0.45 * sin(uT * (3.0 + h * 6.0) + h * 40.0)) : 1.0;
    float gate = mode == 3.0 ? step(0.42, h) : mode == 2.0 ? step(0.56, h) : step(0.38, h);
    float part = exp(-max(d, 0.0) * (mode == 2.0 ? 80.0 : 260.0)) * gate;
    acc += mix(col, col2, h) * part * amt * (0.55 - 0.16 * fl) * flick;
  }
  return acc;
}

vec3 fireflies(vec2 uv, float amt, vec3 col, vec3 col2, float seed){
  vec3 acc = vec3(0.0);
  for (int L = 0; L < 2; L++){
    float fl = float(L);
    float density = 10.0 + fl * 8.0;
    vec2 g = uv * density;
    vec2 id = floor(g);
    vec2 fr = fract(g) - 0.5;
    float h = n21(id + seed + fl * 23.0);
    vec2 wander = 0.30 * vec2(
      sin(uT * (0.3 + h) + h * 50.0) + 0.5 * sin(uT * (0.7 + h * 2.0)),
      cos(uT * (0.26 + h * 0.8) + h * 31.0));
    float d = length(fr - wander);
    float blink = 0.35 + 0.65 * pow(0.5 + 0.5 * sin(uT * (1.2 + h * 2.4) + h * 70.0), 2.0);
    float fly = exp(-d * d * 320.0) * step(0.4, h) * blink;
    acc += mix(col, col2, h) * fly * amt * (0.6 - 0.2 * fl);
  }
  return acc;
}

vec3 drops(vec2 uv, float amt, vec3 col, vec3 hot, float speed, float seed, float cols, float ang, float len){
  vec3 acc = vec3(0.0);
  for (int L = 0; L < 2; L++){
    float fl = float(L);
    float cc = cols + fl * cols * 0.7;
    float cid = floor(uv.x * cc);
    float h = n21(vec2(cid, seed + fl * 31.0));
    float h2 = n21(vec2(cid * 3.7, seed * 2.0 + fl));
    if (h2 < 0.42) continue;
    vec2 dir = vec2(sin(ang), -cos(ang));
    vec2 perp = vec2(-dir.y, dir.x);
    float rel = dot(uv - 0.5, dir) + 0.5;
    float dur = 1.4 / (speed * (0.6 + h));
    float ph = fract(h * 9.7 + uT / dur);
    float dropY = 1.12 - ph * 1.24;
    vec2 op = vec2((cid + 0.35 + 0.3 * h2) / cc, dropY);
    vec2 dvec = uv - op;
    float along = dot(dvec, dir);
    float across = dot(dvec, perp);
    float head = exp(-pow(across / 0.0035, 2.0) - pow(along / 0.007, 2.0));
    float streak = exp(-pow(across / 0.0022, 2.0)) * exp(-max(-along, 0.0) / len) * smoothstep(0.004, 0.02, -along);
    acc += col * streak * 0.55 * amt + hot * head * amt * 1.2;
  }
  return acc;
}

void main(){
  vec2 uv = gl_FragCoord.xy / uRes;
  vec3 bg = texture(uScene, uv).rgb;
  vec4 idol = texture(uIdol, uv);
  vec3 col = bg * (1.0 - idol.a * 0.94) + idol.rgb;

  float asp = uRes.x / uRes.y;
  if (uPartMode > 0.5 && uPartAmt > 0.002){
    vec2 puv = uv;
    if (uPartMode < 1.5)      col += rainGlyphs(puv, uPartAmt, uPartCol, uPartCol2, 0.16, uSeed, 44.0);
    else if (uPartMode < 2.5) col += flutter(puv, uPartAmt, uPartCol, uPartCol2, 0.05, uSeed, 2.0);
    else if (uPartMode < 3.5) col += flutter(puv, uPartAmt, uPartCol, uPartCol2, 0.06, uSeed, 3.0);
    else if (uPartMode < 4.5) col += flutter(puv, uPartAmt, uPartCol, uPartCol2, 0.035, uSeed, 4.0);
    else if (uPartMode < 5.5) col += drops(puv, uPartAmt, uPartCol * 0.85, vec3(1.0), 0.9, uSeed, 40.0, 0.0, 0.10);
    else if (uPartMode < 6.5) col += drops(puv, uPartAmt, uPartCol, uPartCol2, 1.6, uSeed, 30.0, 0.55, 0.05);
    else if (uPartMode < 7.5) col += flutter(puv, uPartAmt, uPartCol, uPartCol2, 0.012, uSeed, 7.0);
    else                      col += fireflies(puv, uPartAmt, uPartCol, uPartCol2, uSeed);
  }
  O = vec4(col, 1.0);
}`;

// ------------------------------------------------------------------- post ----

export const BRIGHT_FRAG = `#version 300 es
precision highp float;
out vec4 O;
uniform sampler2D uTex;
uniform vec2 uRes;
uniform float uThresh;
void main(){
  vec2 uv = gl_FragCoord.xy / uRes;
  vec3 c = texture(uTex, uv).rgb;
  float l = dot(c, vec3(0.2126, 0.7152, 0.0722));
  float k = smoothstep(uThresh, uThresh + 0.4, l);
  O = vec4(c * k, 1.0);
}`;

export const BLUR_FRAG = `#version 300 es
precision highp float;
out vec4 O;
uniform sampler2D uTex;
uniform vec2 uRes;
uniform vec2 uDir;
void main(){
  vec2 uv = gl_FragCoord.xy / uRes;
  float w0 = 0.227027, w1 = 0.194594, w2 = 0.121621, w3 = 0.054054, w4 = 0.016216;
  vec3 c = texture(uTex, uv).rgb * w0;
  c += texture(uTex, uv + uDir * 1.0).rgb * w1;
  c += texture(uTex, uv - uDir * 1.0).rgb * w1;
  c += texture(uTex, uv + uDir * 2.0).rgb * w2;
  c += texture(uTex, uv - uDir * 2.0).rgb * w2;
  c += texture(uTex, uv + uDir * 3.0).rgb * w3;
  c += texture(uTex, uv - uDir * 3.0).rgb * w3;
  c += texture(uTex, uv + uDir * 4.0).rgb * w4;
  c += texture(uTex, uv - uDir * 4.0).rgb * w4;
  O = vec4(c, 1.0);
}`;

export const POST_FRAG = `#version 300 es
precision highp float;
out vec4 O;
uniform sampler2D uScene;
uniform sampler2D uBloom;
uniform vec2 uRes;
uniform float uBloomAmt, uStreak, uChroma, uVig, uGrain, uScan, uSharp;
uniform float uFlash, uFade, uExposure, uSat, uFrame;
uniform vec3 uFlashCol;
${COMMON}
vec3 aces(vec3 x){
  return clamp((x * (2.51 * x + 0.03)) / (x * (2.43 * x + 0.59) + 0.14), 0.0, 1.0);
}
void main(){
  vec2 uv = gl_FragCoord.xy / uRes;
  vec2 c = uv - 0.5;
  float r2 = dot(c, c);
  vec2 duv = 0.5 + c * (1.0 + 0.05 * r2);
  vec2 ca = c * uChroma * 0.006;
  vec2 tx = 1.0 / uRes;
  vec3 col;
  col.r = texture(uScene, duv + ca).r;
  col.g = texture(uScene, duv).g;
  col.b = texture(uScene, duv - ca).b;
  // unsharp mask: pull back edge contrast the bloom softened
  vec3 nb = (texture(uScene, duv + vec2(tx.x, 0.0)).rgb + texture(uScene, duv - vec2(tx.x, 0.0)).rgb
           + texture(uScene, duv + vec2(0.0, tx.y)).rgb + texture(uScene, duv - vec2(0.0, tx.y)).rgb) * 0.25;
  col += (col - nb) * uSharp;
  vec3 bloom = texture(uBloom, duv).rgb;
  vec3 streak = vec3(0.0);
  for (int i = -4; i <= 4; i++){
    streak += texture(uBloom, duv + vec2(float(i) * 0.011, 0.0)).rgb;
  }
  streak *= 0.111;
  col += bloom * uBloomAmt;
  col += streak * uStreak * vec3(0.75, 0.85, 1.15);
  col *= uExposure;
  col = aces(col);
  float l = dot(col, vec3(0.2126, 0.7152, 0.0722));
  col = mix(vec3(l), col, uSat);
  col *= 1.0 - uScan * 0.5 * (0.5 + 0.5 * sin(gl_FragCoord.y * 3.14159));
  col *= mix(1.0, smoothstep(0.75, 0.16, r2), uVig);
  float g = n21(uv * uRes * 0.5 + vec2(uFrame * 17.13, uFrame * 31.7)) - 0.5;
  col += g * uGrain;
  col *= uFade;
  col += uFlashCol * uFlash;
  O = vec4(col, 1.0);
}`;
