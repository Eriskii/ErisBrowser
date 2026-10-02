from pathlib import Path
import hashlib,json,mmap,shutil,struct,subprocess
out=Path(__file__).resolve().parent
source=Path('/tmp/eris-vulkan-raster-prototype/target-198/debug/eris-browser')
final=out/'eris-browser-native-wide';assert not final.exists()
def digest(p):
 h=hashlib.sha256()
 with p.open('rb') as f:
  for block in iter(lambda:f.read(1024*1024),b''):h.update(block)
 return {'bytes':p.stat().st_size,'sha256':h.hexdigest()}
def allocated(p):
 with p.open('rb') as f, mmap.mmap(f.fileno(),0,access=mmap.ACCESS_READ) as data:
  assert data[:6]==b'\x7fELF\x02\x01'
  shoff=struct.unpack_from('<Q',data,40)[0]
  entsize,count,strings=struct.unpack_from('<HHH',data,58)
  assert entsize==64 and 0<count<4096 and strings<count
  rows=[struct.unpack_from('<IIQQQQIIQQ',data,shoff+i*entsize) for i in range(count)]
  names=rows[strings]; namesbytes=data[names[4]:names[4]+names[5]]
  result=[]
  for row in rows:
   if row[2]&2:
    name=namesbytes[row[0]:].split(b'\0',1)[0].decode('ascii')
    payload=b'' if row[1]==8 else data[row[4]:row[4]+row[5]]
    result.append({'name':name,'type':row[1],'flags':row[2],'address':row[3],'size':row[5],'alignment':row[8],'sha256':hashlib.sha256(payload).hexdigest()})
  return {'machine':struct.unpack_from('<H',data,18)[0],'entry':struct.unpack_from('<Q',data,24)[0],'sections':result}
before=digest(source); sections=allocated(source)
shutil.copyfile(source,final);final.chmod(0o755)
subprocess.run(['strip','--strip-debug',str(final)],check=True)
assert allocated(final)==sections
record={'build_log':'native-build-1980.log','source':before,'stripped':digest(final),'allocated_sections_unchanged':True,'allocated_section_count':len(sections['sections']),'allocated':sections,'action':'strip --strip-debug only; not a release/performance build'}
(out/'binary.json').write_text(json.dumps(record,indent=2)+'\n')
print(json.dumps({k:v for k,v in record.items() if k!='allocated'}))
