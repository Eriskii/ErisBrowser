import hashlib,json,os,shutil,subprocess,sys
from pathlib import Path
root=Path('/home/eriskii/Documents/Programming/Projects/ErisBrowser')
out=Path(__file__).resolve().parent
sys.path.insert(0,str(root/'tools'))
from native_raster_timing_host import compiled_sources
before=compiled_sources(root)
(out/'release-source-before.json').write_text(json.dumps(before,indent=2)+'\n')
result=subprocess.run([sys.executable,str(out/'check.py'),'release-build-1','1.98.0','Cargo.toml','vulkan-raster','build','--release','--bin','eris-browser'],cwd=root,close_fds=True)
if result.returncode: raise SystemExit(result.returncode)
after=compiled_sources(root)
assert before==after,'runtime source changed during build'
source=Path('/tmp/eris-vulkan-raster-prototype/target-198/release/eris-browser')
binary=out/'eris-browser-native-reuse-1'
assert not binary.exists()
shutil.copyfile(source,binary);binary.chmod(0o755)
record=dict(binary=str(binary),bytes=binary.stat().st_size,sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),profile='release',toolchain='1.98.0',features=['vulkan-raster'],source_files=after)
(out/'release-1.json').write_text(json.dumps(record,indent=2)+'\n')
print(json.dumps({k:v for k,v in record.items() if k!='source_files'}),flush=True)
