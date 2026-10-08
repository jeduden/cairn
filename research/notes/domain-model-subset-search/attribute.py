"""Attribute findings to features by locating each quote in the annotated sources.
usage: attribute.py <candidate_dir> <findings.json> -> prints per-feature counts, writes attributed.json"""
import re, json, os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from assemble import TOK
GA=os.path.dirname(os.path.abspath(__file__))
def spans_at(text):
    """Map plain-text offset -> innermost feature stack, for marker-free text of an annotated file."""
    plain=[]; stack=[]; owner=[]; pos=0
    for m in TOK.finditer(text):
        seg=text[pos:m.start()]
        plain.append(seg); owner.extend([tuple(stack)]*len(seg))
        if m.group(1): stack.append(m.group(1))
        else: stack.pop()
        pos=m.end()
    seg=text[pos:]; plain.append(seg); owner.extend([tuple(stack)]*len(seg))
    return ''.join(plain), owner
def norm(s): return re.sub(r'\s+',' ',s).strip()
def main(cand, fpath):
    findings=json.load(open(fpath))
    srcs={}
    for f in os.listdir(f'{GA}/annotated'):
        plain,owner=spans_at(open(f'{GA}/annotated/{f}').read())
        # whitespace-normalised index
        idx=[]; out=[]
        prev_space=False
        for i,ch in enumerate(plain):
            if ch.isspace():
                if prev_space: continue
                out.append(' '); idx.append(i); prev_space=True
            else: out.append(ch); idx.append(i); prev_space=False
        srcs[f[:-len('.ann')] if f.endswith('.ann') else f]=(''.join(out), idx, owner)
    res=[]; counts={}
    for fd in findings:
        q=norm(fd.get('quote','')).replace('`','`')
        where=None
        for f,(txt,idx,owner) in srcs.items():
            k=txt.find(q) if q else -1
            for n in (40,25):
                if k<0 and len(q)>n: k=txt.find(q[:n])
                if k<0 and len(q)>n:
                    k2=txt.find(q[-n:]); k=k2 if k2>=0 else k
            if k>=0:
                st=owner[idx[k]]
                where=(f, st[-1] if st else 'core'); break
        feat=where[1] if where else 'unlocated'
        fd['feature']=feat; fd['file']=where[0] if where else None
        res.append(fd)
        if fd.get('severity')=='needs-fix': counts[feat]=counts.get(feat,0)+1
    json.dump(res, open(os.path.join(cand,'attributed.json'),'w'), indent=1)
    print(json.dumps(counts))
if __name__=='__main__': main(sys.argv[1], sys.argv[2])
