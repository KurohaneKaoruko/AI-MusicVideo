import json, numpy as np
import matplotlib; matplotlib.use("Agg")
import matplotlib.pyplot as plt
J=json.load(open("out/analysis.json",encoding="utf-8"))
fps=J["fps"]; per=J["beat_period"]; t0=J["beat_t0"]
o=np.array(J["onset"]); k=np.array(J["kick"]); r=np.array(J["rms"]); h=np.array(J["hat"])
wins=[(8.5,16.5),(88,96),(130,138),(176,182)]
fig,axes=plt.subplots(len(wins),1,figsize=(19,2.5*len(wins)))
fig.patch.set_facecolor("#0d0d0f")
for ax,(a,b) in zip(axes,wins):
    i0,i1=int(a*fps),int(b*fps); x=np.arange(i0,i1)/fps
    ax.set_facecolor("#0d0d0f")
    ax.plot(x,o[i0:i1],color="#5aa9ff",lw=0.9,label="onset")
    ax.plot(x,k[i0:i1],color="#ffa63c",lw=1.3,alpha=.95,label="kick 28-190Hz")
    ax.plot(x,h[i0:i1]*0.5,color="#79ff9c",lw=0.7,alpha=.7,label="hat")
    ax.plot(x,r[i0:i1]*0.45,color="#9aa",lw=0.6,alpha=.5,label="rms")
    n0=int(np.ceil((a-t0)/per)); n1=int(np.floor((b-t0)/per))
    for n in range(n0,n1+1):
        t=t0+n*per; db=(n%4==0)
        ax.axvline(t,color="#ff2d63" if db else "#ff2d63",lw=1.1 if db else .6,alpha=.95 if db else .35)
    ax.set_ylim(-0.05,1.3); ax.set_xlim(a,b)
    ax.set_xticks(np.arange(round(a),b+0.01,0.5))
    ax.tick_params(colors="#c9c9c9",labelsize=7)
    ax.grid(axis="x",color="#2a2a30",lw=.4)
    lg=ax.legend(loc="upper right",fontsize=7,facecolor="#1a1a20",labelcolor="#e0e0e0",ncol=4)
    for s in ax.spines.values(): s.set_color("#3a3a42")
plt.tight_layout(); plt.savefig("out/gridcheck.png",dpi=115,facecolor="#0d0d0f")
print("ok")
