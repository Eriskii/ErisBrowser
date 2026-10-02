from pathlib import Path
from fractions import Fraction
import hashlib,json,re,struct,tarfile
ROOT=Path('/tmp/eris-vulkan-glyph-fixtures')
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
q=lambda n:Fraction(str(n))
def bounds(r,offset=(0,0)):
 x,y,w,h=map(q,r);return x+offset[0],y+offset[1],x+offset[0]+w,y+offset[1]+h
def intersection(a,b):return max(a[0],b[0]),max(a[1],b[1]),min(a[2],b[2]),min(a[3],b[3])
def inside(x,y,b):return b[0]<=x<b[2] and b[1]<=y<b[3]
def arithmetic(c):
 f=c['frame'];w,h=f['width'],f['height'];target=[f['clear']]*(w*h)
 caller=intersection(bounds(f['caller_clip']),(0,0,w,h));clip=caller;offset=tuple(map(q,f['document_offset']));stack=[]
 for op in c['commands']:
  k=op['kind']
  if k=='PushClip':stack.append((k,clip,offset));clip=intersection(clip,bounds(op['rect'],offset));continue
  if k=='PushFixed':stack.append((k,clip,offset));clip=caller;offset=tuple(map(q,f['viewport_offset']));continue
  if k.startswith('Pop'):
   tag,clip,offset=stack.pop();assert tag=='Push'+k[3:];continue
  for y in range(h):
   for x in range(w):
    if not inside(x,y,clip):continue
    if k=='Glyph':
     m=c['masks'][op['source']];sy=y-op['y']
     if not 0<=sy<m['height']:continue
     sx=x-c['row_tables'][op['rows']][sy]
     if not 0<=sx<m['width']:continue
     coverage=m['coverage'][sy*m['width']+sx];color=op['rgba']
    else:
     assert k in ['Rect','Image'];b=bounds(op['rect'],offset)
     assert all(v.denominator==1 for v in b)
     if not inside(x,y,b):continue
     coverage=255
     if k=='Rect':color=op['rgba']
     else:
      im=c['images'][op['source']];assert(im['width'],im['height'])==(1,1);color=im['rgba']
    alpha=(coverage*color[3])//255;old=target[y*w+x];channels=[]
    for src,shift in zip(color[:3],[16,8,0]):
     dest=(old>>shift)&255;channels.append((src*alpha+dest*(255-alpha)+127)//255)
    target[y*w+x]=(channels[0]<<16)|(channels[1]<<8)|channels[2]
 assert not stack
 assert target==[int(v,16) for v in c['expected_packed_rgb']],c['name']
 raw=bytes(v for n in target for v in [(n>>16)&255,(n>>8)&255,n&255]);assert raw==(ROOT/c['expected_file']).read_bytes();assert hashlib.sha256(raw).hexdigest()==c['expected_rgb24_sha256']
 return {'name':c['name'],'pixels':len(target),'rgb24_sha256':hashlib.sha256(raw).hexdigest()}

# Small source-data parser, not a Rust evaluator. Only the declared fixture
# constructors/arrays/scalars are accepted; all bytes must be consumed.
class RustData:
 def __init__(self,path):
  s=Path(path).read_text();s=s[s.index('pub fn fixtures'):];s=s[s.index('{')+1:];s=s.replace('.into()','')
  self.tokens=re.findall(r'"(?:[^"\\]|\\.)*"|(?:[A-Za-z_][A-Za-z0-9_]*::)*[A-Za-z_][A-Za-z0-9_]*|-?(?:0x[0-9a-fA-F]+|[0-9]+)|[^\s]',s);self.at=0;self.bits=[]
 def pop(self,want=None):
  t=self.tokens[self.at];self.at+=1
  if want is not None:assert t==want,(t,want,self.tokens[self.at:self.at+5])
  return t
 def many(self,end):
  v=[]
  while self.tokens[self.at]!=end:
   v.append(self.value())
   if self.tokens[self.at]!=end:self.pop(',')
  self.pop(end);return v
 def value(self):
  t=self.pop()
  if t=='[':return self.many(']')
  if t=='(':return self.many(')')
  if t=='vec':self.pop('!');self.pop('[');return self.many(']')
  if t.startswith('"'):
   s=t[1:-1];return re.sub(r'\\u\{([0-9a-f]+)\}|\\(["\\])',lambda m:chr(int(m[1],16)) if m[1] else m[2],s)
  if t in ('true','false'):return t=='true'
  if re.fullmatch(r'-?(?:0x[0-9a-fA-F]+|[0-9]+)',t):return int(t,16 if '0x' in t else 10)
  if self.tokens[self.at]=='{':
   self.pop('{');d={'$':t}
   while self.tokens[self.at]!='}':
    k=self.pop();self.pop(':');d[k]=self.value()
    if self.tokens[self.at]!='}':self.pop(',')
   self.pop('}');return d
  if self.tokens[self.at]=='(':
   self.pop('(');args=self.many(')')
   if t=='f32::from_bits':
    assert len(args)==1;self.bits.append(args[0]);return struct.unpack('<f',struct.pack('<I',args[0]))[0]
   if t=='Color::rgba':assert len(args)==4;return args
   if t in ('Arc::new','ImageStore::from'):assert len(args)==1;return args[0]
   assert t in ('Command::PushClip',),t
   return {'$':t,'rect':args[0]}
  assert t in ('Command::PushFixed','Command::PopFixed','Command::PopClip','DrawCommand::PushFixed','DrawCommand::PopFixed','DrawCommand::PopClip'),t
  return {'$':t}
 def read(self):
  v=self.value();self.pop('}');assert self.at==len(self.tokens);return v

def rect(r):
 assert r['$']=='Rect';return[r[k] for k in ('x','y','width','height')]
def frame(f):
 assert f['$'] in ('Frame','FontFrame');return{k:(rect(v) if k=='caller_clip' else v) for k,v in f.items() if k!='$'}
def command(c):
 k=c['$'].split('::')[-1];out={'kind':k}
 for name,val in c.items():
  if name=='$':continue
  out['rgba' if name=='color' else name]=rect(val) if name=='rect' else val
 return out
f32=lambda v:struct.unpack('<f',struct.pack('<f',v))[0]
def expected_frame(f):return {k:([f32(n) for n in v] if isinstance(v,list) else v) for k,v in f.items()}
def expected_command(c):
 out={k:v for k,v in c.items() if k not in ('f32_bits','unicode_scalars')}
 for k in (('x','y','size') if c['kind']=='Text' else ('radius',)):
  if k in out:out[k]=f32(out[k])
 if 'rect' in out:out['rect']=[f32(v) for v in out['rect']]
 return out

doc=json.loads((ROOT/'literal-fixtures.json').read_text());rows=[arithmetic(c) for c in doc['pixel_cases']];assert sum(r['pixels'] for r in rows)==223
assert 8+512+262144*4+512*4==1051144
assert 53*76800==4070400 and 614400+307200+960+53*256==936128
assert 52*76800==3993600 and 614400+307200+960+52*256==935872
lit=RustData(ROOT/'literal_inventory.rs');cases=lit.read();assert len(cases)==12
for actual,c in zip(cases,doc['pixel_cases']):
 assert actual['$']=='LiteralFixture';assert actual['name']==c['name'];assert frame(actual['frame'])==expected_frame(c['frame'])
 assert [command(v) for v in actual['commands']]==[expected_command(v) for v in c['commands']]
 assert [{k:v for k,v in m.items() if k!='$'} for m in actual['masks']]==c['masks']
 assert [{k:v for k,v in m.items() if k!='$'} for m in actual['images']]==c['images']
 assert actual['rows']==c['row_tables'];assert actual['expected']==[int(v,16) for v in c['expected_packed_rgb']]
fontdoc=json.loads((ROOT/'font-inventory.json').read_text());font=RustData(ROOT/'font_inventory.rs');fontcases=font.read();assert len(fontcases)==14
for actual,c in zip(fontcases,fontdoc['cases']):
 assert actual['$']=='FontFixture';assert actual['name']==c['name'];assert frame(actual['frame'])==expected_frame(c['frame'])
 assert [command(v) for v in actual['commands']]==[expected_command(v) for v in c['commands']]
 images=[{'key':key,**{k:v for k,v in im.items() if k!='$'}} for key,im in actual['images']];assert images==c['images']
 for command_json in c['commands']:
  if command_json['kind']=='Text':
   assert [f'U+{ord(ch):04X}' for ch in command_json['text']]==command_json['unicode_scalars']
   for k in ('x','y','size'):assert struct.pack('<f',command_json[k]).hex()==bytes.fromhex(command_json['f32_bits'][k])[::-1].hex()
ledger=json.loads((ROOT/'parent-source-freeze.json').read_text());assert sha(ROOT/ledger['archive']['path'])==ledger['archive']['sha256']
with tarfile.open(ROOT/'parent-source.tar') as archive:
 for item in ledger['files']:
  data=(ROOT/'reference-parent'/item['path']).read_bytes();assert len(data)==item['bytes'];assert hashlib.sha256(data).hexdigest()==item['sha256'];assert archive.extractfile(item['path']).read()==data
paths=['prepare.py','literal-fixtures.json','literal_inventory.rs','font-inventory.json','font_inventory.rs','parent-source-freeze.json','reference-parent/src/bin/glyph-reference.rs']
result={'scope':'Read-only independent arithmetic/data/source review, no candidate/font/planner/Canvas execution','input_hashes':{str(ROOT/p):sha(ROOT/p) for p in paths},'literal_rows':rows,'literal_rust_exact':True,'font_input_rust_exact':True,'font_inputs':14,'all_parent_files_verified':len(ledger['files']),'cap_arithmetic':True,'findings':[],'notes':['Destination-oriented per-cell arithmetic independently recomputed all223 pixels; exact integer endpoint clipping covers all supplied half-integer/integer geometry.','Rust source-data decoder compared every fixture field, source cell, row origin, command, literal target and source f32 value, without compiling or evaluating Rust.','Harness source statically checked: preserved parent-only eris dependency, fresh output directory/create_new files, independent fresh Fonts per fixture, two original Canvas passes compared cold/warm, exhausted/high-byte refusal, every pixel written u32 little-endian.','No actual-font pixels have been produced or judged by this review.','Preparation-only checks stopped first on the in-flight expected_rgb24 field rename, then on this independent decoder treating literal Glyph.y as f32. Type-aware decoding fixed the checker; no fixture/pixel/oracle bytes changed or mismatch was normalized.']}
out=Path('/tmp/eris-vulkan-glyph-fixtures-peer-review.json');assert not out.exists();out.write_text(json.dumps(result,indent=2)+'\n');print(out,sha(out))
