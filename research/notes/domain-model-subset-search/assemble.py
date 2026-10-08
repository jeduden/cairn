"""Assemble a candidate model from annotated sources and a genome.

Annotated sources live in annotated/<name>.md.ann (the ten concept files, the
hub as index.md, and invariants.md). A span ⟦f:ID⟧ ... ⟦/f⟧ belongs to feature
ID; spans nest. A span of a feature not in the genome is removed with its
contents. The core feature is always present.

usage: assemble.py <genome.json> <outdir>
genome.json: {"name": "...", "features": ["id", ...]}
"""
import re, json, os, sys
GA=os.path.dirname(os.path.abspath(__file__))
TOK=re.compile(r'⟦f:([a-z0-9-]+)⟧|⟦/f⟧')

def strip(text, keep):
    out=[]; stack=[]; pos=0; removed_terms=[]; drop_depth=None
    buf_removed=[]
    for m in TOK.finditer(text):
        seg=text[pos:m.start()]
        if drop_depth is None: out.append(seg)
        else: buf_removed.append(seg)
        if m.group(1):
            stack.append(m.group(1))
            if drop_depth is None and m.group(1) not in keep:
                drop_depth=len(stack)
        else:
            if not stack: raise ValueError('unbalanced close at %d' % m.start())
            stack.pop()
            if drop_depth is not None and len(stack) < drop_depth:
                drop_depth=None
        pos=m.end()
    seg=text[pos:]
    if drop_depth is None: out.append(seg)
    else: buf_removed.append(seg)
    if stack: raise ValueError('unclosed spans: %s' % stack)
    removed=''.join(buf_removed)
    removed_terms=re.findall(r'\*\*([^*]+)\*\*', removed)
    return ''.join(out), [t.rstrip(':').strip() for t in removed_terms]

def tidy(text):
    lines=[]
    for l in text.split('\n'):
        if re.match(r'^\s*-\s*$', l): continue          # emptied list item
        if l and not l.strip(): continue                  # indentation left by a removed item
        l=re.sub(r'[ \t]{2,}', ' ', l) if l.strip() and not l.startswith('  ') else re.sub(r'(?<=\S)[ \t]{2,}', ' ', l)
        l=re.sub(r'\s+([,.;:)])', r'\1', l)
        l=re.sub(r'\(\s*\)', '', l)
        l=re.sub(r',\s*,', ',', l); l=re.sub(r';\s*;', ';', l)
        l=re.sub(r'\(\s*,\s*', '(', l); l=re.sub(r',\s*\)', ')', l)
        lines.append(l)
    # drop blank lines a removed list item left between two list lines
    keep=[]
    for i,l in enumerate(lines):
        if l=='' and keep and i+1<len(lines):
            prev=keep[-1]; nxt=lines[i+1]
            if re.match(r'^\s*- ', nxt) and (re.match(r'^\s*- ', prev) or prev.startswith(' ')):
                continue
        keep.append(l)
    lines=keep
    t='\n'.join(lines)
    t=re.sub(r'\n{3,}', '\n\n', t)
    return t

def closure(texts, removed_terms, core_terms):
    """Removed bold terms still mentioned in the kept text (dangling references)."""
    found=[]
    body='\n'.join(texts.values())
    IGNORE={'review step','leave','repository','reply','assignment','result','overlap','span','stat','outcome','publish','pick','tree','states'}
    for term in sorted(set(removed_terms)):
        if term.lower() in core_terms or term.lower().strip('`') in IGNORE: continue
        t=re.sub(r'`', '', term)
        if len(t) < 3: continue
        pat=r'(?<![\w-])'+re.escape(t)+r'(s|es)?(?![\w-])'
        for name, txt in texts.items():
            for mm in re.finditer(pat, txt, re.I):
                line=txt[:mm.start()].count('\n')+1
                found.append({'file':name,'line':line,'term':term})
    return found

def main(genome_path, outdir):
    g=json.load(open(genome_path))
    keep=set(g['features'])|{'core'}
    src=os.path.join(GA,'annotated')
    os.makedirs(outdir, exist_ok=True)
    texts={}; removed=[]
    for f in sorted(os.listdir(src)):
        # <name>.md.ann keeps mdsmith off the markers; plain .md still reads
        name=f[:-len('.ann')] if f.endswith('.md.ann') else f
        if not name.endswith('.md'): continue
        t,rt=strip(open(os.path.join(src,f)).read(), keep)
        texts[name]=tidy(t); removed+=rt
    # regenerate the hub's catalog from the assembled files' front matter
    if 'index.md' in texts:
        rows=[]
        for f in sorted(texts):
            if f in ('index.md','invariants.md'): continue
            fm=re.match(r'^---\n(.*?)\n---', texts[f], re.S)
            if not fm: continue
            title=re.search(r'title:\s*"([^"]*)"', fm.group(1))
            summ=re.search(r'summary:\s*>-\s*\n((?:\s+.*\n?)*)', fm.group(1)+'\n')
            st=' '.join(summ.group(1).split()) if summ else ''
            rows.append(f'- [{title.group(1) if title else f}]({f}) — {st}')
        texts['index.md']=re.sub(r'<\?catalog.*?<\?/catalog\?>', '\n'.join(rows), texts['index.md'], flags=re.S)
    kept_terms=set()
    for t in texts.values():
        kept_terms|={x.rstrip(':').strip().lower() for x in re.findall(r'\*\*([^*]+)\*\*', t)}
    dangling=closure(texts, removed, kept_terms)
    for f,t in texts.items(): open(os.path.join(outdir,f),'w').write(t)
    order=['invariants.md','index.md']+sorted(k for k in texts if k not in ('invariants.md','index.md'))
    with open(os.path.join(outdir,'candidate.md'),'w') as fh:
        for f in order:
            if f in texts: fh.write(f'\n\n<!-- FILE: {f} -->\n\n'+texts[f])
    json.dump({'genome':g,'dangling':dangling,'chars':sum(len(t) for t in texts.values())}, open(os.path.join(outdir,'assembly.json'),'w'), indent=1)
    print(g.get('name'), 'features', len(keep), 'chars', sum(len(t) for t in texts.values()), 'dangling', len(dangling))
    return dangling

if __name__=='__main__':
    main(sys.argv[1], sys.argv[2])
