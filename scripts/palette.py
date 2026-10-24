#!/usr/bin/env python3
"""Print the sRGB hex table and key contrast ratios for spec/09 §2.6 from the OKLCH ramps in §2.1.
Usage: scripts/palette.py   (no dependencies; paste the output into spec/09 §2.6)"""
import math
def oklch_to_lin(L,C,H):
    a=C*math.cos(math.radians(H)); b=C*math.sin(math.radians(H))
    l_=L+0.3963377774*a+0.2158037573*b; m_=L-0.1055613458*a-0.0638541728*b; s_=L-0.0894841775*a-1.2914855480*b
    l,m,s=l_**3,m_**3,s_**3
    return (4.0767416621*l-3.3077115913*m+0.2309699292*s, -1.2684380046*l+2.6097574011*m-0.3413193965*s, -0.0041960863*l-0.7034186147*m+1.7076147010*s)
def ingamut(rgb): return all(-1e-6<=c<=1+1e-6 for c in rgb)
def gamma_encode(c):
    c=min(max(c,0),1); return 12.92*c if c<=0.0031308 else 1.055*c**(1/2.4)-0.055
def to_hex(L,C,H):
    lo,hi=0,C
    if not ingamut(oklch_to_lin(L,C,H)):
        for _ in range(40):
            mid=(lo+hi)/2
            if ingamut(oklch_to_lin(L,mid,H)): lo=mid
            else: hi=mid
        C=lo
    r,g,b=[gamma_encode(c) for c in oklch_to_lin(L,C,H)]
    return "#%02X%02X%02X"%tuple(round(x*255) for x in (r,g,b)), C
def lum(hexs):
    v=[int(hexs[i:i+2],16)/255 for i in (1,3,5)]
    v=[c/12.92 if c<=0.04045 else ((c+0.055)/1.055)**2.4 for c in v]
    return 0.2126*v[0]+0.7152*v[1]+0.0722*v[2]
def cr(a,b):
    la,lb=sorted([lum(a),lum(b)],reverse=True); return (la+0.05)/(lb+0.05)
steps=[50,100,200,300,400,500,600,700,800,900,950]
neutral=[(0.985,0.003),(0.965,0.005),(0.925,0.008),(0.870,0.010),(0.710,0.015),(0.550,0.018),(0.450,0.020),(0.370,0.020),(0.280,0.018),(0.210,0.015),(0.150,0.012)]
brand=[(0.97,0.02),(0.93,0.04),(0.87,0.07),(0.78,0.11),(0.66,0.16),(0.56,0.19),(0.49,0.20),(0.42,0.18),(0.35,0.14),(0.28,0.10),(0.20,0.07)]
accent=[(0.98,0.03,75),(0.95,0.06,75),(0.90,0.10,75),(0.85,0.14,75),(0.80,0.16,75),(0.75,0.16,70),(0.67,0.15,65),(0.56,0.13,60),(0.46,0.10,55),(0.38,0.08,55),(0.27,0.06,55)]
P={}
P['gray']=[to_hex(L,C,255)[0] for L,C in neutral]
P['brand']=[to_hex(L,C,255)[0] for L,C in brand]
P['accent']=[to_hex(L,C,H)[0] for L,C,H in accent]
for name,h in [('green',150),('amber',75),('red',25),('blue',240),('violet',300)]:
    P[name]=[to_hex(L,min(C,0.17),h)[0] for L,C in brand]
print("| Step | "+" | ".join(P.keys())+" |"); print("|---"*(len(P)+1)+"|")
for i,s in enumerate(steps): print(f"| {s} | "+" | ".join(f"`{P[k][i]}`" for k in P)+" |")
g=dict(zip(steps,P['gray'])); br=dict(zip(steps,P['brand']))
def st(n,s): return dict(zip(steps,P[n]))[s]
W="#FFFFFF"
pairs=[("Light","--color-text on --color-bg",g[900],g[50]),("Light","--color-text-muted on --color-surface",g[600],W),("Light","--color-text-muted on --color-bg",g[600],g[50]),("Light","--color-text-subtle on --color-surface",g[500],W),
("Light","--color-primary-fg (white) on --color-primary",W,br[600]),("Light","--color-primary on --color-surface",br[600],W),("Light","link --color-primary on --color-bg",br[600],g[50]),
("Light","white on --color-success",W,st('green',600)),("Light","white on --color-warning (D-72)",W,st('amber',600)),("Light","white on --color-danger",W,st('red',600)),("Light","white on --color-info",W,st('blue',600)),
("Light","--color-danger on --color-danger-subtle",st('red',600),st('red',50)),("Light","--color-border-input on --color-surface (D-73)",g[500],W),
("Dark","--color-text on --color-bg",g[50],g[950]),("Dark","--color-text-muted on --color-surface",g[400],g[900]),("Dark","brand-950 on --color-primary",br[950],br[400]),("Dark","--color-primary on --color-surface",br[400],g[900]),
("Dark","--color-danger on --color-surface",st('red',400),g[900]),("Dark","--color-border-input on --color-surface (D-73)",g[500],g[900])]
print(); print("| Mode | Pair | Fg | Bg | Ratio |"); print("|---|---|---|---|---|")
for m,n,a,b in pairs: print(f"| {m} | {n} | `{a}` | `{b}` | {cr(a,b):.2f}:1 |")
