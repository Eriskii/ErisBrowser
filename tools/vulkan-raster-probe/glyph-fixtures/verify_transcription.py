#!/usr/bin/env python3
"""Decode the restricted Rust data-expression syntax, without Rust execution."""
import hashlib,json,re,struct
from pathlib import Path
D=Path(__file__).resolve().parent

def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
class Parser:
 def __init__(self,s):
  self.s=s;self.i=0
  self.ts=[]
  pat=re.compile(r'\s+|//[^\n]*|"(?:\\.|[^"\\])*"|[A-Za-z_][A-Za-z_0-9]*(?:::[A-Za-z_][A-Za-z_0-9]*)*|0x[0-9a-fA-F_]+|[0-9]+|[{}\[\](),:.!\-]')
  while self.i<len(s):
   m=pat.match(s,self.i);assert m,repr(s[self.i:self.i+50]);self.i=m.end();t=m.group()
   if not t.isspace() and not t.startswith('//'):self.ts.append(t)
  self.i=0
 def pop(self,t=None):
  q=self.ts[self.i];self.i+=1
  if t is not None:assert q==t,(q,t)
  return q
 def peek(self):return self.ts[self.i] if self.i<len(self.ts) else None
 def sequence(self,end):
  a=[]
  while self.peek()!=end:
   a.append(self.expr())
   if self.peek()==',':self.pop(',')
   else:break
  self.pop(end);return a
 def expr(self):
  t=self.pop()
  if t=='-':return -self.expr()
  if t=='vec':self.pop('!');self.pop('[');return self.sequence(']')
  if t=='[':return self.sequence(']')
  if t=='(':return tuple(self.sequence(')'))
  if t.startswith('"'):
   q=json.loads(re.sub(r'\\u\{([0-9a-fA-F]+)\}',lambda m:chr(int(m[1],16)),t))
   if self.peek()=='.':self.pop('.');self.pop('into');self.pop('(');self.pop(')')
   return q
  if t in ('true','false'):return t=='true'
  if t[0].isdigit():return int(t.replace('_',''),0) if t.startswith('0x') else int(t)
  if self.peek()=='(':
   self.pop('(');a=self.sequence(')')
   if t=='f32::from_bits':return struct.unpack('<f',struct.pack('<I',a[0]))[0]
   return dict(type=t,args=a)
  d=dict(type=t)
  if self.peek()=='{':
   self.pop('{')
   while self.peek()!='}':
    k=self.pop();self.pop(':');d[k]=self.expr()
    if self.peek()==',':self.pop(',')
    else:break
   self.pop('}')
  return d

def rustdata(p):
 s=p.read_text();start=s.index('{',s.index('pub fn fixtures()'))+1;s=s[start:s.rindex('}')];q=Parser(s);x=q.expr();assert q.peek() is None;return x

def float32(v):return struct.unpack('<f',struct.pack('<f',v))[0]
def rect(r):return [r[k] for k in ('x','y','width','height')]
def frame(r):return dict(width=r['width'],height=r['height'],clear=r['clear'],caller_clip=rect(r['caller_clip']),document_offset=list(r['document_offset']),viewport_offset=list(r['viewport_offset']))
def command(r):
 k=r['type'].split('::')[-1];c=dict(kind=k)
 if k=='PushClip':c['rect']=rect(r['args'][0] if 'args' in r else r['rect'])
 elif k in ('Rect','Image'):
  c['rect']=rect(r['rect'])
  if 'rgba' in r:c['rgba']=r['rgba']
  if 'color' in r:c['rgba']=r['color']['args']
  for key in ('source','key','radius'):
   if key in r:c[key]=r[key]
 elif k=='Glyph':c.update({q:r[q] for q in ('source','rows','y','rgba')})
 elif k=='Text':
  c.update({q:r[q] for q in ('text','x','y','size','bold','italic','monospace')});c['rgba']=r['color']['args']
 return c

def qjson(c):
 c=json.loads(json.dumps(c))
 for cmd in c['commands']:
  cmd.pop('f32_bits',None);cmd.pop('unicode_scalars',None)
  for k in (('x','y','size') if cmd['kind']=='Text' else ('radius',)):
   if k in cmd:cmd[k]=float32(cmd[k])
  if 'rect' in cmd:cmd['rect']=[float32(v) for v in cmd['rect']]
 for k in ('caller_clip','document_offset','viewport_offset'):c['frame'][k]=[float32(v) for v in c['frame'][k]]
 return c

lit=json.loads((D/'literal-fixtures.json').read_text())['pixel_cases'];actual=rustdata(D/'literal_inventory.rs');assert len(lit)==len(actual)==12
for frozen,r in zip(lit,actual):
 c=qjson(frozen)
 got=dict(name=r['name'],frame=frame(r['frame']),commands=[command(x) for x in r['commands']],images=[{k:x[k] for k in ('width','height','rgba')} for x in r['images']],masks=[{k:x[k] for k in ('width','height','coverage')} for x in r['masks']],row_tables=r['rows'])
 assert got=={k:c[k] for k in got},frozen['name']
 assert r['expected']==[int(v,16) for v in c['expected_packed_rgb']]
 rgb=(D/c['expected_file']).read_bytes();assert len(rgb)==c['expected_rgb24_bytes'];assert hashlib.sha256(rgb).hexdigest()==c['expected_rgb24_sha256']
 assert rgb==bytes(v for px in r['expected'] for v in ((px>>16)&255,(px>>8)&255,px&255))
fonts=json.loads((D/'font-inventory.json').read_text())['cases'];actual=rustdata(D/'font_inventory.rs');assert len(fonts)==len(actual)==14
for frozen,r in zip(fonts,actual):
 c=qjson(frozen);ims=[]
 for key,arc in r['images']['args'][0]:
  x=arc['args'][0];ims.append(dict(key=key,**{k:x[k] for k in ('width','height','rgba')}))
 got=dict(name=r['name'],frame=frame(r['frame']),commands=[command(x) for x in r['commands']],images=ims)
 assert got=={k:c[k] for k in got},frozen['name']
# Check archived parent inputs, without invoking its code or any build tooling.
ledger=json.loads((D/'parent-source-freeze.json').read_text())
for row in ledger['files']:
 p=D/'reference-parent'/row['path'];assert sha(p)==row['sha256'];assert p.stat().st_size==row['bytes']
assert sha(D/'parent-source.tar')==ledger['archive']['sha256']
receipt=dict(schema=1,method='Restricted Rust literal parser independently reconstitutes all 26 fixture names/frames/commands/source and row tables; compares every scalar (including exact f32 bit values), Unicode string, image key, expected pixel and RGB24 byte to frozen JSON. No Rust/compiler/renderer/planner/font APIs executed.',literal_cases=12,literal_pixels=sum(len(x['expected_packed_rgb']) for x in lit),font_cases=14,font_outputs_produced=0,parent_files_exact=len(ledger['files']),files=[dict(path=p.relative_to(D).as_posix(),bytes=p.stat().st_size,sha256=sha(p)) for p in [D/'literal-fixtures.json',D/'font-inventory.json',D/'literal_inventory.rs',D/'font_inventory.rs',D/'reference-parent/src/bin/glyph-reference.rs',D/'prepare.py',Path(__file__)]] )
(D/'transcription-review.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({k:receipt[k] for k in ('literal_cases','literal_pixels','font_cases','font_outputs_produced','parent_files_exact')}))
