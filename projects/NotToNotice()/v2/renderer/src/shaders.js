// ---------------------------------------------------------------------------
// shaders.js - every GLSL program in the film
// ---------------------------------------------------------------------------

// uFlipY is 0 for everything on screen; the capture path sets it to 1 on the
// final post pass only, because gl.readPixels hands back rows bottom-up.
export const VS_QUAD = `#version 300 es
layout(location=0) in vec2 aPos;
out vec2 vUv;
uniform float uFlipY;
void main(){
  vUv = aPos*0.5+0.5;
  float y = aPos.y * (uFlipY > 0.5 ? -1.0 : 1.0);
  gl_Position = vec4(aPos.x, y, 0.0, 1.0);
}`;

// ---------------------------------------------------------------- nebula ----
// Rendered at 1/16 resolution: it is a low-frequency field, so upscaling is
// visually lossless and this is where nearly all the per-pixel cost lives.

export const FS_NEBULA = `#version 300 es
precision highp float;
in vec2 vUv;
out vec4 outColor;
uniform float uT, uScale, uWarp, uFlow, uContrast, uGain, uAspect;
uniform vec3 uC0, uC1, uC2;
uniform vec2 uOff;

float h(vec2 p){
  p = fract(p*vec2(123.34, 456.21));
  p += dot(p, p+45.32);
  return fract(p.x*p.y);
}
float n2(vec2 p){
  vec2 i = floor(p), f = fract(p);
  f = f*f*(3.0-2.0*f);
  return mix(mix(h(i), h(i+vec2(1,0)), f.x),
             mix(h(i+vec2(0,1)), h(i+vec2(1,1)), f.x), f.y);
}
float fbm(vec2 p){
  float s=0.0, a=0.5;
  for(int i=0;i<4;i++){ s += a*n2(p); p = p*2.03 + 17.1; a *= 0.5; }
  return s;
}
void main(){
  vec2 p = (vUv*2.0-1.0);
  p.x *= uAspect;
  p = p*uScale + uOff;
  vec2 q = vec2(fbm(p + uT*uFlow*0.35),
                fbm(p + vec2(5.2,1.3) - uT*uFlow*0.28));
  float f = fbm(p + uWarp*q + uT*uFlow*0.10);
  float g = fbm(p*2.07 - q*1.35 + uT*uFlow*0.05);
  vec3 col = mix(uC0, uC1, smoothstep(0.18, 0.82, f));
  col = mix(col, uC2, smoothstep(0.42, 1.0, g)*mix(0.25, 0.85, uContrast));
  col *= uGain * (0.30 + 1.45*pow(smoothstep(0.0, 0.80, f), 1.25));
  outColor = vec4(col, 1.0);
}`;

// ------------------------------------------------------------ background ----
// Full resolution composite: upscaled nebula + analytic grid + light shafts.
// Everything here is cheap (a few smoothsteps per pixel).

export const FS_BG = `#version 300 es
precision highp float;
in vec2 vUv;
out vec4 outColor;
uniform sampler2D uNeb;
uniform float uT, uAspect, uZoom, uRot, uPanX, uPanY;
uniform vec3 uC0, uC1, uC2, uGridCol, uShaftCol, uGlowCol;
uniform float uGridAmt, uGridMode, uGridScale, uGridDepth, uShaftAmt, uShaftAng, uShaftWide;
uniform float uGlowAmt, uGlowR, uGlowX, uGlowY, uNebMix, uHaze, uFloor;

vec2 cam(vec2 uv){
  vec2 p = uv*2.0-1.0; p.x *= uAspect;
  p -= vec2(uPanX, uPanY);
  float c = cos(uRot), s = sin(uRot);
  p = mat2(c,-s,s,c) * p;
  return p / max(uZoom, 0.001);
}

float gridLines(float v, float w){
  float f = abs(fract(v)-0.5);
  return smoothstep(w, 0.0, f);
}

void main(){
  vec2 uv = vUv;
  vec2 p = cam(uv);

  // ---- nebula (upscaled, parallaxed) ------------------------------------
  vec2 np = uv*0.5 + vec2(0.5) + p*0.018;
  vec3 col = texture(uNeb, np).rgb;
  col = mix(col, vec3(dot(col, vec3(0.30,0.55,0.15))) * mix(uC0*2.4, uC2*1.8, 0.5),
            (1.0-uNebMix)*0.85);

  // ---- horizon / floor ---------------------------------------------------
  float hz = uFloor;
  col *= mix(1.0, 0.22, smoothstep(hz, hz-0.55, p.y));

  // ---- grid --------------------------------------------------------------
  float gm = uGridMode;
  float g = 0.0;
  if(gm > 0.5 && gm < 1.5){
    // perspective floor: rows receding to the horizon, columns spreading
    float y = p.y - hz;
    float z = 1.0/max(y, 0.0015);
    g += gridLines(z*uGridScale*0.35, 0.045) * smoothstep(hz, hz-0.02, p.y);
    float x = p.x/z;
    g += gridLines(x*uGridScale*0.30, 0.035) * smoothstep(hz, hz-0.02, p.y);
    g *= smoothstep(0.0, 0.25, 1.0/(1.0+z*0.02));
    g += pow(max(0.0, 1.0-abs(y)*7.0), 3.0)*0.5;
  } else if(gm > 1.5 && gm < 2.5){
    // polar rings
    float r = length(p);
    float a = atan(p.y, p.x);
    g += gridLines(r*uGridScale*0.55 + uT*0.06, 0.055);
    g += gridLines(a*4.0/3.14159*uGridScale*0.32, 0.030) * smoothstep(2.4, 0.2, r);
  } else if(gm > 2.5 && gm < 3.5){
    // vertical data columns
    g += gridLines(p.x*uGridScale*0.42, 0.055);
    g *= smoothstep(1.6, 0.0, abs(p.y));
  } else if(gm > 3.5 && gm < 4.5){
    // hex-ish lattice
    vec2 q = vec2(p.x*1.0, p.y*1.1547);
    float a1 = abs(fract(q.x*0.5)-0.5);
    float a2 = abs(fract((q.x*0.5 + q.y*0.577))-0.5);
    float a3 = abs(fract((q.x*0.5 - q.y*0.577))-0.5);
    g = smoothstep(0.44, 0.5, max(max(a1,a2),a3));
  }
  g = pow(clamp(g,0.0,1.0), uGridDepth);
  col += uGridCol * g * uGridAmt;

  // ---- volumetric light shafts ------------------------------------------
  if(uShaftAmt > 0.001){
    vec2 lp = vec2(uGlowX, uGlowY) - vec2(uPanX*0.0, 0.0);
    vec2 d = lp - p;
    float ang = atan(d.y, d.x) - uShaftAng;
    float s = 0.0;
    for(int i=0;i<5;i++){
      float fi = float(i);
      float w = 0.030 + fi*0.021;
      float o = fi*0.115;
      s += exp(-pow(abs(sin(ang*1.0+o)),2.0)/(w*w));
    }
    s /= 5.0;
    float fall = exp(-length(d)*mix(1.4, 0.55, uShaftWide));
    col += uShaftCol * s * fall * uShaftAmt;
  }

  // ---- central glow ------------------------------------------------------
  if(uGlowAmt > 0.001){
    float d = length(p - vec2(uGlowX, uGlowY));
    col += uGlowCol * uGlowAmt * exp(-pow(d/max(uGlowR,0.01), 1.55));
  }

  // ---- haze --------------------------------------------------------------
  col = mix(col, uC1*0.5, uHaze * smoothstep(0.0, 1.2, length(p)));

  outColor = vec4(max(col, 0.0), 1.0);
}`;

// ---------------------------------------------------------------- sprites ---
// One shader draws every particle / ring / filament / glyph-block in the film.
// Instance data lives in a 4 x (3N) float texture so we only touch one buffer
// per frame.

export const VS_SPRITE = `#version 300 es
layout(location=0) in vec2 aQuad;
uniform sampler2D uData;
uniform vec2 uRes, uCam;
uniform float uZoom, uRot;
uniform int uCount;
out vec2 vL;
out vec4 vCol;
out vec4 vP;
void main(){
  int i = gl_InstanceID;
  vec4 d0 = texelFetch(uData, ivec2(0, i), 0);
  vec4 d1 = texelFetch(uData, ivec2(4, i), 0);
  vec4 d2 = texelFetch(uData, ivec2(8, i), 0);
  vec2 pos = d0.xy; float size = d0.z; float rot = d0.w;
  vCol = d1;
  vP = d2;

  vec2 local = aQuad * size;
  float c = cos(rot), s = sin(rot);
  vec2 rl = mat2(c,-s,s,c) * local;

  // screen-space: origin at the top-left, y growing downwards, matching the
  // 2D canvas layer so scenes can author both with the same coordinates
  vec2 p = pos - uRes*0.5 - uCam;
  p += rl;
  p /= max(uZoom, 0.001);

  vec2 ndc = p / (uRes*0.5);
  ndc.y = -ndc.y;
  gl_Position = vec4(ndc, 0.0, 1.0);
  vL = aQuad;
}`;

const SDF_LIB = `
float sdBox(vec2 p, vec2 b){ vec2 d = abs(p)-b; return length(max(d,0.0)) + min(max(d.x,d.y),0.0); }
float sdCaps(vec2 p, float h, float r){ p.x = abs(p.x)-h; p.x = max(p.x,0.0); return length(p)-r; }
float sdTri(vec2 p, float r){
  const float k = 1.7320508;
  p.x = abs(p.x) - r;
  p.y = p.y + r/k;
  if(p.x + k*p.y > 0.0) p = vec2(p.x - k*p.y, -k*p.x - p.y)/2.0;
  p.x -= clamp(p.x, -2.0*r, 0.0);
  return -length(p)*sign(p.y);
}
float sdHex(vec2 p, float r){
  const vec3 k = vec3(-0.8660254, 0.5, 0.57735);
  p = abs(p);
  p -= 2.0*min(dot(k.xy,p), 0.0)*k.xy;
  p -= vec2(clamp(p.x, -k.z*r, k.z*r), r);
  return length(p)*sign(p.y);
}
float sdStar(vec2 p, float r, float n, float m){
  float an = 3.14159265/n;
  float en = 3.14159265/m;
  vec2 acs = vec2(cos(an), sin(an));
  vec2 ecs = vec2(cos(en), sin(en));
  float bn = mod(atan(p.x,p.y), 2.0*an) - an;
  p = length(p)*vec2(cos(bn), abs(sin(bn)));
  p -= r*acs;
  p += ecs*clamp(-dot(p,ecs), 0.0, r*acs.y/ecs.y);
  return length(p)*sign(p.x);
}
`;

export const FS_SPRITE = `#version 300 es
precision highp float;
in vec2 vL;
in vec4 vCol;
in vec4 vP;
out vec4 outColor;
uniform float uAA;
${SDF_LIB}
void main(){
  vec2 p = vL;
  float shape = vP.x;
  float p1 = vP.y, p2 = vP.z, p3 = vP.w;
  float a = 0.0;
  vec3 col = vCol.rgb;

  if(shape < 0.5){
    // soft glow disc
    float d = length(p);
    a = exp(-pow(d/max(p1,0.02), 2.0));
    a += 0.28*exp(-pow(d/max(p1*2.3,0.02), 1.5));
  } else if(shape < 1.5){
    // ring / annulus, p1 thickness, p2 softness
    float d = abs(length(p) - 0.72) - p1;
    a = exp(-pow(max(d,0.0)/max(p2,0.04), 1.6));
  } else if(shape < 2.5){
    // capsule / streak : p1 half length, p2 thickness, p3 glow
    float d = sdCaps(p, p1, p2);
    float w = max(p3, 0.01);
    a = exp(-pow(max(d,0.0)/w, 1.7));
    a += 0.30*exp(-pow(abs(d)/max(w*3.0,0.02), 2.0));
  } else if(shape < 3.5){
    // tear / droplet
    vec2 q = p;
    float taper = 1.0 - clamp(-q.y, 0.0, 1.0)*p1;
    q.x /= max(taper, 0.05);
    float d = length(q) - p2;
    a = exp(-pow(max(d,0.0)/max(p3,0.02), 1.5));
    a += 0.35*exp(-pow(abs(d)/max(p3*3.0,0.02), 2.0));
  } else if(shape < 4.5){
    // 5-petal flower (the hanamaru stamp)
    float th = atan(p.y, p.x);
    float r = length(p);
    float pet = p1*(0.66 + 0.34*cos(5.0*th + p2));
    float d = abs(r - pet);
    a = exp(-pow(d/max(p3,0.02), 1.3));
    a += 0.20*exp(-pow(r/max(pet*1.9,0.02), 3.0));
  } else if(shape < 5.5){
    // triangle shard
    float d = sdTri(p, 0.85);
    a = exp(-pow(max(d,0.0)/max(p1,0.02), 1.4));
    a += 0.3*exp(-pow(abs(d)/max(p1*3.0,0.02), 2.0));
  } else if(shape < 6.5){
    // hard block (redaction bars, glyph cells)
    vec2 b = vec2(p1, p2);
    float d = sdBox(p, b);
    a = 1.0 - smoothstep(-uAA, uAA, d);
    a += 0.5*exp(-pow(max(d,0.0)/max(p3,0.02), 1.5));
  } else if(shape < 7.5){
    // hexagon
    float d = sdHex(p, 0.8);
    a = exp(-pow(max(d,0.0)/max(p1,0.02), 1.4));
    a += 0.3*exp(-pow(abs(d)/max(p1*3.0,0.02), 1.8));
  } else {
    // four-point spark
    float d = sdStar(p, 0.88, p1, p2);
    a = exp(-pow(max(d,0.0)/max(p3,0.02), 1.2));
    a += 0.22*exp(-pow(abs(d)/max(p3*3.0,0.02), 2.0));
  }

  a = max(a, 0.0);
  outColor = vec4(col * a * vCol.a, 1.0);
}`;

// Solid (alpha-blended) variant of the same shapes.
export const FS_SPRITE_SOLID = FS_SPRITE
  .replace("outColor = vec4(col * a * vCol.a, 1.0);",
           "outColor = vec4(col, clamp(a,0.0,1.0) * vCol.a);");

// ------------------------------------------------------------------ bloom ---

// generic full-screen composite (used to add the text layer into the scene)
export const FS_TEX = `#version 300 es
precision highp float;
in vec2 vUv;
out vec4 outColor;
uniform sampler2D uTex;
uniform float uGain, uAlpha;
uniform vec3 uMul;
void main(){
  vec4 c = texture(uTex, vUv);
  outColor = vec4(c.rgb * uMul * uGain, c.a * uAlpha);
}`;

export const FS_BRIGHT = `#version 300 es
precision highp float;
in vec2 vUv;
out vec4 outColor;
uniform sampler2D uTex;
uniform float uThresh, uKnee, uIntensity;
void main(){
  vec3 c = texture(uTex, vUv).rgb;
  float l = max(max(c.r,c.g),c.b);
  float s = clamp((l - uThresh) / max(uKnee, 0.0001), 0.0, 1.0);
  s *= s;
  outColor = vec4(c * s * uIntensity, 1.0);
}`;

export const FS_BLUR = `#version 300 es
precision highp float;
in vec2 vUv;
out vec4 outColor;
uniform sampler2D uTex;
uniform vec2 uDir;
void main(){
  // 9-tap gaussian on the linear-filtered pyramid
  vec3 s = texture(uTex, vUv).rgb * 0.227027;
  s += (texture(uTex, vUv + uDir*1.3846).rgb + texture(uTex, vUv - uDir*1.3846).rgb) * 0.316216;
  s += (texture(uTex, vUv + uDir*3.2308).rgb + texture(uTex, vUv - uDir*3.2308).rgb) * 0.070270;
  outColor = vec4(s, 1.0);
}`;

export const FS_UP = `#version 300 es
precision highp float;
in vec2 vUv;
out vec4 outColor;
uniform sampler2D uTex, uAdd;
uniform float uMix;
void main(){
  vec3 a = texture(uTex, vUv).rgb;
  vec3 b = texture(uAdd, vUv).rgb;
  outColor = vec4(a + b*uMix, 1.0);
}`;

// ------------------------------------------------------------------- post ---

export const FS_POST = `#version 300 es
precision highp float;
in vec2 vUv;
out vec4 outColor;
uniform sampler2D uScene, uBloom;
uniform vec2 uRes;
uniform float uT, uFrame;
uniform float uBloomAmt, uChroma, uChromaR, uGlitch, uGlitchSeed;
uniform float uScan, uGrain, uVignette, uFlash, uFade, uSat, uContrast, uLift;
uniform float uSmear, uSmearY, uRows, uExposure;
uniform vec3 uFlashCol, uTint, uLiftCol;

float h21(vec2 p){ return fract(sin(dot(p, vec2(12.9898,78.233)))*43758.5453); }
float h11(float x){ return fract(sin(x*78.233)*43758.5453); }

vec3 scene(vec2 uv){
  return texture(uScene, uv).rgb;
}

void main(){
  vec2 uv = vUv;
  vec2 c = uv - 0.5;

  // ---- datamosh / row displacement --------------------------------------
  if(uGlitch > 0.0001){
    float rows = max(uRows, 4.0);
    float ry = floor(uv.y*rows);
    float r = h21(vec2(ry, floor(uGlitchSeed)));
    if(r > 1.0-uGlitch*0.65){
      float sh = (h11(ry + uGlitchSeed*0.37) - 0.5) * uGlitch * 0.22;
      uv.x += sh;
    }
    float band = h21(vec2(floor(uv.y*7.0), floor(uGlitchSeed*0.5)));
    if(band > 1.0-uGlitch*0.4){
      uv.x += (h11(floor(uv.y*7.0)+uGlitchSeed)-0.5)*uGlitch*0.10;
    }
  }
  if(uSmear > 0.0001){
    float d = smoothstep(uSmearY+0.22, uSmearY-0.22, vUv.y);
    uv.x = mix(uv.x, uv.x + (uv.x-0.5)*0.0, d);
    uv = mix(uv, vec2(0.5) + (uv-0.5)*0.72, d*uSmear);
  }

  // ---- chromatic aberration ---------------------------------------------
  vec3 col;
  if(uChroma > 0.0001){
    vec2 dir = normalize(c + 1e-6);
    vec2 off = dir*uChroma*0.012 + vec2(uChromaR*0.006, 0.0);
    col.r = scene(uv + off).r;
    col.g = scene(uv).g;
    col.b = scene(uv - off).b;
  } else {
    col = scene(uv);
  }

  // ---- bloom -------------------------------------------------------------
  col += texture(uBloom, uv).rgb * uBloomAmt;

  // ---- grade -------------------------------------------------------------
  col *= uExposure;
  col = mix(col, uLiftCol, uLift);
  float lum = dot(col, vec3(0.2126,0.7152,0.0722));
  col = mix(vec3(lum), col, uSat);
  col *= uTint;
  col = (col - 0.5)*uContrast + 0.5;

  // ---- scanlines ---------------------------------------------------------
  if(uScan > 0.0001){
    float s = sin(vUv.y*uRes.y*1.55)*0.5+0.5;
    col *= 1.0 - uScan*(0.30+0.70*s);
  }

  // ---- vignette ----------------------------------------------------------
  float v = length(c*vec2(1.05,1.0));
  col *= mix(1.0, smoothstep(0.95, 0.28, v), uVignette);

  // ---- flash -------------------------------------------------------------
  col += uFlashCol * uFlash;

  // ---- grain (also dithers away banding) ---------------------------------
  if(uGrain > 0.0001){
    float g = h21(vUv*uRes + vec2(uFrame*1.37, uFrame*0.71)) - 0.5;
    float g2 = h21(vUv*uRes*0.5 - vec2(uFrame*2.1, uFrame*1.3)) - 0.5;
    col += (g*0.7 + g2*0.3) * uGrain;
  }

  col *= uFade;
  outColor = vec4(max(col, 0.0), 1.0);
}`;
