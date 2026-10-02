#!/usr/bin/env python3
"""Independent literal integer/fraction arithmetic and data-only Rust transcription.
Never imports a renderer, planner, font library, or candidate module.
"""
import hashlib,json,struct
from fractions import Fraction as F
from pathlib import Path
D=Path(__file__).resolve().parent

def write(name,obj):
 p=D/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(json.dumps(obj,indent=2,ensure_ascii=True)+'\n');return p

def frame(w,h,clear=0xffffff,clip=None,doc=(0,0),fixed=(0,0)):
 return dict(width=w,height=h,clear=clear,caller_clip=clip or [0,0,w,h],document_offset=list(doc),viewport_offset=list(fixed))
def mask(w,h,c):return dict(width=w,height=h,coverage=c)
def glyph(s,r,y,c):return dict(kind='Glyph',source=s,rows=r,y=y,rgba=c)
def rect(r,c):return dict(kind='Rect',rect=r,rgba=c,radius=0)
def push(r):return dict(kind='PushClip',rect=r)
def simple(k):return dict(kind=k)
def case(name,fr,ms,rows,cmds,images=None,why=''):
 return dict(name=name,frame=fr,masks=ms,row_tables=rows,images=images or [],commands=cmds,claim=why)
cs=[]
cs.append(case('coverage-alpha-floor',frame(6,4,0x204060),[mask(6,1,[0,1,127,128,254,255])],[[0]],
 [glyph(0,0,y,[231,93,17,a]) for y,a in enumerate([255,128,1,254])],why='Coverage alpha truncates before each independently rounded channel blend.'))
cs.append(case('ordered-overlap',frame(4,3,0x102030),[mask(2,2,[255,128,64,0])],[[0,1],[1,0]],
 [glyph(0,0,0,[255,0,0,128]),glyph(0,1,1,[0,255,0,192]),glyph(0,0,0,[0,0,255,64])]))
cs.append(case('repeated-low-alpha',frame(1,1,0),[mask(1,1,[255])],[[0]],
 [glyph(0,0,0,[255,255,255,1]) for _ in range(8)],why='Eight individually rounded alpha-one operations; final 0x080808.'))
cs.append(case('interleaved-rect-image',frame(4,2,0),[mask(2,2,[255,128,64,255])],[[0,1],[2,2]],
 [rect([0,0,4,2],[255,0,0,255]),glyph(0,0,0,[0,0,255,128]),dict(kind='Image',source=0,rect=[1,0,2,2]),glyph(0,1,0,[255,255,0,192])],
 [dict(width=1,height=1,rgba=[0,255,0,64])]))
cs.append(case('signed-row-origins-and-y',frame(4,3,0xeeeeee),[mask(3,3,[255,128,64,32,255,128,64,32,255])],[[-2,0,2]],
 [glyph(0,0,-1,[13,117,233,223])],why='Negative y clips the first mask row, without advancing or flattening row origins.'))
cs.append(case('varying-row-origins',frame(4,4,0),[mask(3,3,[255,128,64,32,255,128,64,32,255])],[[2,-1,1]],
 [glyph(0,0,1,[250,111,27,255])]))
cs.append(case('fractional-clip-both-edges',frame(5,3,0xffffff,[0.5,0.5,3,1.5]),[mask(5,3,[255]*15)],[[0,0,0]],
 [glyph(0,0,0,[255,0,0,128])],why='Only integer origins x=1,2,3 and y=1 satisfy the fractional clip; y=2 is excluded.'))
cs.append(case('fixed-escape-and-restoration',frame(6,4,0xffffff,[1,0,4,4],(1,-1),(2,1)),[mask(2,2,[255]*4)],[[1,1],[3,3],[4,4]],
 [push([0,1,2,2]),glyph(0,0,0,[255,0,0,255]),simple('PushFixed'),glyph(0,1,1,[0,0,255,255]),simple('PopFixed'),glyph(0,2,0,[0,255,0,255]),simple('PopClip'),glyph(0,2,2,[0,255,0,255])],
 why='Glyph positions are absolute even inside Fixed. Fixed escapes inner clip and Pop restores it; final PopClip restores caller clip.'))
cs.append(case('absolute-origins-ignore-offset',frame(5,3,0,[0,0,5,3],(1,2),(-2,-1)),[mask(2,1,[255,128])],[[0],[2]],
 [glyph(0,0,0,[255,0,0,255]),simple('PushFixed'),glyph(0,1,1,[0,255,0,255]),simple('PopFixed')]))
cs.append(case('empty-zero-coverage-and-alpha',frame(3,2,0x123456),[mask(0,0,[]),mask(2,2,[0]*4),mask(1,1,[255])],[[],[0,0],[1]],
 [glyph(0,0,0,[255,0,0,255]),glyph(1,1,0,[255,0,0,255]),glyph(2,2,0,[255,0,0,0]),push([0,0,0,0]),glyph(2,2,0,[255,0,0,255]),simple('PopClip')],
 why='Canonical empty has no dispatch; nonempty zero coverage still dispatches; alpha zero/empty clip omit only after validation.'))
cs.append(case('signed-i32-extremes',frame(3,3,0x010203),[mask(3,3,[255]*9)],[[2147483647,-2147483648,0]],
 [glyph(0,0,0,[0,255,0,255]),glyph(0,0,2147483647,[255,0,0,255]),glyph(0,0,-2147483648,[0,0,255,255])],
 why='Only the final row of first operation is visible; signed origin/extent arithmetic must not wrap.'))
cs.append(case('dispatch-tail-9x9',frame(9,9,0x334455),[mask(9,9,[(x*31+y*17)%256 for y in range(9) for x in range(9)])],[[0]*9],
 [glyph(0,0,0,[215,87,39,173])],why='Partial 8x8 workgroups must not write beyond the target or source.'))

# Integer/fraction model only: fixtures contain no font outlines or candidate-derived coverage.
def intersect(a,b):
 x=max(a[0],b[0]);y=max(a[1],b[1]);return [x,y,max(x,min(a[0]+a[2],b[0]+b[2]))-x,max(y,min(a[1]+a[3],b[1]+b[3]))-y]
def frac(r):return [F(v) for v in r]
def inside(x,y,r):return r[2]>0 and r[3]>0 and r[0]<=x<r[0]+r[2] and r[1]<=y<r[1]+r[3]
def blend(old,c,cov):
 a=c[3]*cov//255
 return sum((((c[i]*a+((old>>s)&255)*(255-a)+127)//255)<<s) for i,s in enumerate((16,8,0)))
def literal(c):
 fr=c['frame'];w,h=fr['width'],fr['height'];p=[fr['clear']]*(w*h)
 caller=intersect(frac(fr['caller_clip']),[0,0,w,h]);clip=caller;offset=fr['document_offset'];stack=[];steps=[]
 for cmd in c['commands']:
  k=cmd['kind'];touched=[]
  if k=='PushClip':
   stack.append(('clip',clip,offset));r=frac(cmd['rect']);r[0]+=offset[0];r[1]+=offset[1];clip=intersect(clip,r)
  elif k=='PushFixed':stack.append(('fixed',clip,offset));clip=caller;offset=fr['viewport_offset']
  elif k in ('PopFixed','PopClip'):
   expect='fixed' if k=='PopFixed' else 'clip';tag,clip,offset=stack.pop();assert tag==expect
  elif k=='Glyph':
   m=c['masks'][cmd['source']];rows=c['row_tables'][cmd['rows']];assert len(rows)==m['height']
   for gy in range(m['height']):
    for gx in range(m['width']):
     x=rows[gy]+gx;y=cmd['y']+gy
     if 0<=x<w and 0<=y<h and inside(x,y,clip):
      i=y*w+x;p[i]=blend(p[i],cmd['rgba'],m['coverage'][gy*m['width']+gx]);touched.append(i)
  else:
   assert k in ('Rect','Image');r=frac(cmd['rect']);r[0]+=offset[0];r[1]+=offset[1]
   for y in range(h):
    for x in range(w):
     if inside(x,y,clip) and inside(x,y,r):
      color=cmd['rgba'] if k=='Rect' else c['images'][cmd['source']]['rgba'];assert len(color)==4
      i=y*w+x;p[i]=blend(p[i],color,255);touched.append(i)
  steps.append(dict(kind=k,touched_pixel_indices=touched,packed_rgb=p.copy()))
 assert not stack
 return p,steps
for c in cs:
 p,steps=literal(c);rgb=bytes(v for q in p for v in ((q>>16)&255,(q>>8)&255,q&255))
 c['expected_packed_rgb']=[f'{q:06x}' for q in p];c['expected_rgb24_bytes']=len(rgb);c['expected_rgb24_sha256']=hashlib.sha256(rgb).hexdigest();c['expected_file']='literal-rgb/'+c['name']+'.rgb24'
 (D/c['expected_file']).parent.mkdir(exist_ok=True);(D/c['expected_file']).write_bytes(rgb)
 c['arithmetic_steps']=steps
assert cs[2]['expected_packed_rgb']==['080808']

caps=dict(source_entries=256,row_tables=256,row_table_entries=1024,total_row_entries=65536,commands=256,scopes=32,mask_axis=1024,mask_area=262144,total_coverage_bytes=262144,gpu_buffer_bytes=1048576,invocations=4000000,width=320,height=240)
neg=[]
def refuse(name,family,input_recipe,reason,variants=None):neg.append(dict(name=name,expect='refuse',family=family,input_recipe=input_recipe,reason=reason,variants=variants or []))
refuse('source-index','glyph-source-index',{'base':'coverage-alpha-floor','replace_glyph_source':1},'Mask-only source index is outside the single-mask table.',['alpha0','empty_clip'])
refuse('row-index','row-table-index',{'base':'coverage-alpha-floor','replace_glyph_rows':1},'Row-table-only index is outside the single table.',['alpha0','empty_clip'])
refuse('row-height','row-height-mismatch',{'frame':[1,1],'mask':[1,2,{'fill':255,'count':2}],'rows':[[0]],'glyph':{'source':0,'rows':0,'y':0}},'Reached row table length must equal mask height.',['alpha0','empty_clip'])
for shape in ([0,1],[1,0]):refuse('noncanonical-empty-'+str(shape[0]),'mask-empty-shape',{'mask':shape+[[]],'commands':[]},'Only 0x0 with empty coverage is valid empty.')
refuse('coverage-length','mask-byte-length',{'mask':[2,2,[255,255,255]],'commands':[]},'Unused malformed source is rejected before any dispatch.')
for axis in ('width','height'):refuse('axis-'+axis,'mask-axis',{'mask':([1025,1] if axis=='width' else [1,1025])+[{'fill':0,'count':1025}],'commands':[]},'One mask axis exceeds 1024 even unused.')
refuse('mask-area','mask-area',{'mask':[513,512,{'fill':0,'count':262656}],'commands':[]},'Each axis fits, but product exceeds 262144.')
refuse('aggregate-coverage','mask-coverage-total',{'masks':[[512,512,{'fill':0,'count':262144}],[1,1,[0]]],'commands':[]},'Valid individual shapes total 262145 coverage bytes.')
refuse('combined-sources','combined-source-count',{'images':[[1,1,[0,0,0,0]]],'masks':{'repeat':[0,0,[]],'count':256},'commands':[]},'Image and mask sources together total 257.')
refuse('row-table-count','row-table-count',{'rows':{'repeat':[],'count':257},'commands':[]},'257 supplied empty tables exceed table count.')
refuse('row-table-size','row-table-length',{'rows':[{'fill':0,'count':1025}],'commands':[]},'Unused row table exceeds length 1024.')
refuse('aggregate-rows','row-entry-total',{'rows':[{'repeat_table':{'fill':0,'count':1024},'count':64},[0]],'commands':[]},'65 tables individually fit but total 65537 entries.')
refuse('command-count','operation-count',{'commands':{'repeat':{'kind':'Glyph','source':0,'rows':0,'y':0,'rgba':[0,0,0,0]},'count':257},'masks':[[0,0,[]]],'rows':[[]]},'Transparent empty glyph operations still count before omission.')
refuse('scope-depth','scope-depth',{'commands':[{'repeat':'PushFixed','count':33},{'repeat':'PopFixed','count':33}]},'Balanced nesting depth 33 exceeds 32 independently of op count.')
refuse('gpu-buffer-cap','explicit-gpu-bytes',{'frame':[1,1],'masks':[[512,512,{'fill':255,'count':262144}]],'rows':[{'fill':0,'count':512}],'glyph':{'source':0,'rows':0,'y':0}},'Visible glyph packs all coverage as u32: 1048576 coverage + 2048 rows + 512 uniforms + 8 target/readback = 1051144 > 1048576. No images/LUTs.')
refuse('invocation-cap','gpu-invocations',{'frame':[320,240],'masks':[[320,240,{'fill':255,'count':76800}]],'rows':[{'fill':0,'count':240}],'glyph_repeat':52},'Unconditional clear plus 52 full-frame glyph draws: 53*76800 = 4070400 > 4000000. Explicit bytes 614400+307200+960+53*256=936128 stay below cap.')
refuse('target-high-byte','opaque-target',{'frame':[1,1],'clear':4278190080,'commands':[]},'Packed target clear must be 0x00RRGGBB, independent of text alpha.')

literal_doc=dict(schema=1,classification='independent-literal-mask-oracle',pixel_file_encoding='RGB24: three bytes R,G,B per row-major pixel; JSON packed hex is 0x00RRGGBB',method='Standalone integer/Fraction arithmetic over supplied literal coverage; no renderer, planner, fonts or engine imported/executed.',alpha_rule='a=floor(color_alpha*coverage/255); channel=floor((src*a+dst*(255-a)+127)/255), independently for every ordered draw',coordinate_rule='Glyph row origins and y absolute signed i32; never offset again. Current clip applies at integer pixel origins with exclusive upper edges.',caps=caps,pixel_cases=cs,refusal_cases=neg,positive_cap_plans=[dict(name='invocation-near-cap',recipe='invocation-cap with glyph_repeat=51',invocations=3993600,gpu_buffer_bytes=935872,expected='accept'),dict(name='exact-row-total',recipe='64 row tables each length 1024, no commands or masks',row_entries=65536,expected='accept'),dict(name='exact-combined-source-count',recipe='256 canonical empty masks, no commands or rows',expected='accept')])
write('literal-fixtures.json',literal_doc)

# Actual-font input-only lists; output remains unknown until immutable parent is run.
def text(s,x=3,y=2,size=16,color=(20,40,90,255),bold=False,italic=False,mono=False):
 return dict(kind='Text',text=s,x=x,y=y,size=size,rgba=list(color),bold=bold,italic=italic,monospace=mono)
def fontcase(name,cmds,why,fr=None,images=None):return dict(name=name,frame=fr or frame(128,48),commands=cmds,images=images or [],classification='reference-derived-parent-CPU',reference_status='not executed',purpose=why)
fs=[]
fs.append(fontcase('font-regular-kern-av-to',[text('AV To VA')],'Audited regular AV -131 and To -348 font-unit kerning.'))
fs.append(fontcase('font-bold-translucent',[text('AV To',color=(230,40,90,128),bold=True)],'Bold face identity and audited kerning with translucent text.',frame(128,48,0x204060)))
fs.append(fontcase('font-mono-no-kern',[text('AV To',mono=True)],'Mono no kern table and constant 1233-unit advances.'))
fs.append(fontcase('font-mono-bold-precedence',[text('AV To',mono=True,bold=True)],'Same input as mono case; monospace wins bold.'))
fs.append(fontcase('font-italic-negative-bearing',[text('jTy g',x=-0.5,y=1.5,size=18,italic=True,color=(120,15,220,191))],'Synthetic italic has row-dependent rounded x; j and T negative bearings.'))
fs.append(fontcase('font-space-zero-advance',[text('A \u200dV  A',size=17)],'Regular space is empty with positive advance; U+200D empty with zero advance.'))
fs.append(fontcase('font-missing-regular',[text('A\U0010ffff\u4e2dV')],'Both absent cmap scalars select regular glyph0; no Unicode font fallback.'))
fs.append(fontcase('font-missing-bold',[text('A\U0010ffffV',bold=True)],'Absent scalar selects bold glyph0.'))
fs.append(fontcase('font-missing-mono',[text('A\u200d\U0010ffffV',mono=True)],'Mono U+200D is absent, not empty; both missing scalars select glyph0.'))
fs.append(fontcase('font-composite-unicode',[text('é e\u0301 Ω Ж',size=18)],'Composite é, combining mark and present Greek/Cyrillic map IDs; no shaping claim.'))
fs.append(fontcase('font-subpixel-rounding',[text('jA',x=-0.5,y=-0.5,color=(255,0,0,255)),text('jA',x=31.5,y=0.5,color=(0,0,255,128)),text('jA',x=63.499996185302734,y=1.4999998807907104,color=(0,128,0,255))],'Signed half-away rounding and immediately-below positive f32 boundary.'))
fs.append(fontcase('font-shared-bitmap-quantum',[text('AVTo',size=16.01,y=0),text('AVTo',size=16.04,y=24,color=(190,20,40,255))],'Both bitmap sizes quantize to 16; original-size advances/kerning still differ.'))
fs.append(fontcase('font-bitmap-quantum-boundary',[text('Tj',size=16.062498092651367,x=0),text('Tj',size=16.0625,x=36),text('Tj',size=16.062501907348633,x=72)],'Three neighboring f32 sizes straddle (size*8).round cache/raster boundary.'))
fs.append(fontcase('font-fixed-clip-image-order',[
 rect([0,0,128,48],[20,30,40,255]),push([0,0,40,28]),text('AVAV',x=0,y=0,color=(255,30,30,128)),simple('PushFixed'),text('jTo',x=40,y=6,italic=True,color=(20,240,80,192)),dict(kind='Image',key='dot',rect=[46,12,12,8]),simple('PopFixed'),text('AVAV',x=0,y=10,color=(30,40,255,128)),simple('PopClip'),text('AV',x=94,y=28,color=(250,240,0,255))
 ],'Ordered text/image overlap, caller clip, document offset, fixed escape and restoration.',frame(128,48,0xffffff,[1,1,125,46],(3,2),(0,0)),[dict(key='dot',width=1,height=1,rgba=[255,255,255,64])]))
for c in fs:
 for cmd in c['commands']:
  if cmd['kind']=='Text':
   cmd['f32_bits']={k:f'{struct.unpack("<I",struct.pack("<f",cmd[k]))[0]:08x}' for k in ('x','y','size')}
   cmd['unicode_scalars']=[f'U+{ord(ch):04X}' for ch in cmd['text']]
meta=json.loads(Path('/tmp/eris-vulkan-font-metadata-audit/facts.json').read_text())
fontdoc=dict(schema=1,parent='5ad2cfd3582f92f8368a0a0fa2fd3bb7dd7af2f0',classification='reference-derived-parent-CPU-baselines',claim='Input inventory only. Future immutable-parent CPU pixels test equivalent font raster/placement on GPU; they are not independent glyph coverage goldens.',reference_status='not executed',baseline_file_encoding='Four-byte little-endian 0x00RRGGBB per row-major pixel; <name>.rgb, no header',baseline_bytes_per_case=128*48*4,case_count=len(fs),cache_policy='One fresh Fonts per case; render twice with same Fonts, require cold/warm exact equality, retain one packed-u32le frame file. No output generated during preparation.',font_facts_sha256=hashlib.sha256(Path('/tmp/eris-vulkan-font-metadata-audit/facts.json').read_bytes()).hexdigest(),fonts=[{k:f[k] for k in ('font','bytes','sha256','units_per_em','glyph_count')} for f in meta['fonts']],cases=fs)
write('font-inventory.json',fontdoc)

# Literal data transcription into public parent types. No rendering or coverage evaluation here.
def f32(v):return 'f32::from_bits(0x'+struct.pack('>f',v).hex()+')'
def rr(v):return 'Rect { x: '+f32(v[0])+', y: '+f32(v[1])+', width: '+f32(v[2])+', height: '+f32(v[3])+' }'
def col(v):return 'Color::rgba('+', '.join(map(str,v))+')'
def ruststr(s):return '"'+''.join(('\\"' if ch=='"' else '\\\\' if ch=='\\' else ch if 32<=ord(ch)<127 else '\\u{'+format(ord(ch),'x')+'}') for ch in s)+'"'
def cmdrust(c):
 k=c['kind']
 if k=='Text':return 'DrawCommand::Text { x: '+f32(c['x'])+', y: '+f32(c['y'])+', text: '+ruststr(c['text'])+'.into(), size: '+f32(c['size'])+', color: '+col(c['rgba'])+', bold: '+str(c['bold']).lower()+', italic: '+str(c['italic']).lower()+', monospace: '+str(c['monospace']).lower()+' }'
 if k=='Rect':return 'DrawCommand::Rect { rect: '+rr(c['rect'])+', color: '+col(c['rgba'])+', radius: '+f32(c['radius'])+' }'
 if k=='Image':return 'DrawCommand::Image { rect: '+rr(c['rect'])+', key: '+ruststr(c['key'])+'.into() }'
 if k=='PushClip':return 'DrawCommand::PushClip { rect: '+rr(c['rect'])+' }'
 return 'DrawCommand::'+k
out=['// Data-only transcription of font-inventory.json; no expected raster pixels.','use eris::graphics::{Color, DrawCommand, ImageStore, RasterImage, Rect};','use std::sync::Arc;','pub struct FontFrame { pub width:u32, pub height:u32, pub clear:u32, pub caller_clip:Rect, pub document_offset:(f32,f32), pub viewport_offset:(f32,f32) }','pub struct FontFixture { pub name: &\'static str, pub frame:FontFrame, pub commands:Vec<DrawCommand>, pub images:ImageStore }','pub fn fixtures() -> Vec<FontFixture> { vec![']
for c in fs:
 fr=c['frame']; ims='ImageStore::from(['+', '.join('('+ruststr(im['key'])+'.into(), Arc::new(RasterImage { width:'+str(im['width'])+', height:'+str(im['height'])+', rgba:vec!'+repr(im['rgba'])+' }))' for im in c['images'])+'])'
 out+=['FontFixture { name:'+ruststr(c['name'])+', frame:FontFrame { width:'+str(fr['width'])+', height:'+str(fr['height'])+', clear:0x'+format(fr['clear'],'06x')+', caller_clip:'+rr(fr['caller_clip'])+', document_offset:('+','.join(f32(v) for v in fr['document_offset'])+'), viewport_offset:('+','.join(f32(v) for v in fr['viewport_offset'])+') }, commands:vec!['+',\n'.join(cmdrust(cmd) for cmd in c['commands'])+'], images:'+ims+' },']
out+= ['] }']
(D/'font_inventory.rs').write_text('\n'.join(out)+'\n')
write('preparation-summary.json',dict(literal_cases=len(cs),literal_pixels=sum(c['frame']['width']*c['frame']['height'] for c in cs),literal_rgb_bytes=sum(c['expected_rgb24_bytes'] for c in cs),semantic_refusals=len(neg),positive_cap_plans=3,font_cases=len(fs),font_pixels=sum(c['frame']['width']*c['frame']['height'] for c in fs),font_outputs_produced=0,engines_fonts_planners_renderers_executed=False))
# Pure literal data for the new probe checker; construction calls no planner.
def lowcmd(c):
 k=c['kind']
 if k=='Glyph':return 'Command::Glyph { source:'+str(c['source'])+', rows:'+str(c['rows'])+', y:'+str(c['y'])+', rgba:'+repr(c['rgba'])+' }'
 if k=='Rect':return 'Command::Rect { rect:'+rr(c['rect'])+', rgba:'+repr(c['rgba'])+', radius:'+f32(c['radius'])+' }'
 if k=='Image':return 'Command::Image { rect:'+rr(c['rect'])+', source:'+str(c['source'])+' }'
 if k=='PushClip':return 'Command::PushClip('+rr(c['rect'])+')'
 return 'Command::'+k
ls=['// Trusted bounded literal inputs and independent full packed-RGB expectations.','use crate::{Command, Frame, Rect};','pub struct OwnedImage { pub width:u32, pub height:u32, pub rgba:Vec<u8> }','pub struct OwnedMask { pub width:u32, pub height:u32, pub coverage:Vec<u8> }',"pub struct LiteralFixture { pub name:&'static str, pub frame:Frame, pub commands:Vec<Command>, pub images:Vec<OwnedImage>, pub masks:Vec<OwnedMask>, pub rows:Vec<Vec<i32>>, pub expected:Vec<u32> }",'pub fn fixtures() -> Vec<LiteralFixture> { vec![']
for c in cs:
 fr=c['frame']
 ls+=['LiteralFixture { name:'+ruststr(c['name'])+', frame:Frame { width:'+str(fr['width'])+', height:'+str(fr['height'])+', clear:0x'+format(fr['clear'],'06x')+', caller_clip:'+rr(fr['caller_clip'])+', document_offset:('+','.join(f32(v) for v in fr['document_offset'])+'), viewport_offset:('+','.join(f32(v) for v in fr['viewport_offset'])+') }, commands:vec!['+',\n'.join(lowcmd(cmd) for cmd in c['commands'])+'], images:vec!['+', '.join('OwnedImage { width:'+str(im['width'])+', height:'+str(im['height'])+', rgba:vec!'+repr(im['rgba'])+' }' for im in c['images'])+'], masks:vec!['+', '.join('OwnedMask { width:'+str(m['width'])+', height:'+str(m['height'])+', coverage:vec!'+repr(m['coverage'])+' }' for m in c['masks'])+'], rows:vec!['+', '.join('vec!'+repr(r) for r in c['row_tables'])+'], expected:vec!['+', '.join('0x'+v for v in c['expected_packed_rgb'])+'] },']
ls+= ['] }']
(D/'literal_inventory.rs').write_text('\n'.join(ls)+'\n')
