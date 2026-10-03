import json, numpy as np
J=json.load(open("out/analysis.json",encoding="utf-8"))
fps=J["fps"]; per=J["beat_period"]; t0=J["beat_t0"]
k=np.array(J["kick"]); o=np.array(J["onset"])
print(f"grid bpm={J['bpm']:.3f} period={per:.5f} t0={t0:.4f}")

def peaks(env, thr, gap):
    idx=np.where((env>thr)&(env>np.roll(env,1))&(env>=np.roll(env,-1)))[0]
    out=[]; last=-99
    for i in idx:
        t=i/fps
        if t-last<gap: continue
        out.append((t,float(env[i]))); last=t
    return out

def scan(pk, label):
    ht=np.array([p[0] for p in pk]); hs=np.array([p[1] for p in pk]); hs/=hs.max()
    best=None
    for off in np.linspace(0,per,2000,endpoint=False):
        d=np.abs(ht-(off+np.round((ht-off)/per)*per))
        sc=float((hs*np.exp(-(d/0.035)**2)).sum()/hs.sum())
        if best is None or sc>best[1]: best=(float(off),sc)
    # also the offset-of-offsets: median signed delta at the chosen phase
    off=best[0]
    d=ht-(off+np.round((ht-off)/per)*per)
    print(f"{label}: n={len(ht)} best_phase={best[0]:.4f} score={best[1]:.3f} "
          f"median|d|={np.median(np.abs(d))*1000:.1f}ms signed={np.median(d)*1000:+.1f}ms")
    return best

for thr,lab in ((0.25,"kick>0.25"),(0.4,"kick>0.40"),(0.6,"kick>0.60")):
    scan(peaks(k,thr,0.12),lab)
for thr,lab in ((0.35,"onset>0.35"),(0.6,"onset>0.60")):
    scan(peaks(o,thr,0.12),lab)

print()
print("phase candidates vs downbeat structure (kick>0.4):")
ht=np.array([p[0] for p in peaks(k,0.4,0.12)])
for off in np.arange(0,per,per/12):
    d=ht-(off+np.round((ht-off)/per)*per)
    sc=np.mean(np.abs(d)<0.05)
    print(f"  phase={off:.4f}  frac within 50ms={sc:.2f}  signed={np.median(d)*1000:+6.1f}ms")
