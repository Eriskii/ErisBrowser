import hashlib,json,os,subprocess,sys,time
from pathlib import Path
ROOT=Path('/home/eriskii/Documents/Programming/Projects/ErisBrowser')
OUT=Path(__file__).resolve().parent
label,toolchain,manifest,features,action,*tail=sys.argv[1:]
env=os.environ.copy()
env.update(CARGO_HOME='/tmp/eris-vulkan-browser-cargo-home',RUSTUP_HOME='/tmp/eris-vulkan-rustup',CARGO_TARGET_DIR='/tmp/eris-vulkan-raster-prototype/target-'+('msrv' if toolchain=='1.88.0' else '198'),CARGO_BUILD_JOBS='4')
env['CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER']='python3 '+str(ROOT/'tools/ci_test_runner.py')
argv=['cargo','+'+toolchain,action,'--locked','--offline','--manifest-path',manifest]
if features!='-':argv+=['--features',features]
argv+=tail
log=OUT/(label+'.log')
started=time.time()
with log.open('xb') as stream:
 result=subprocess.run(argv,cwd=ROOT,env=env,stdout=stream,stderr=subprocess.STDOUT,close_fds=True)
record={'command':argv,'exit':result.returncode,'elapsed_seconds':round(time.time()-started,3),'log':log.name,'sha256':hashlib.sha256(log.read_bytes()).hexdigest()}
(OUT/(label+'.json')).write_text(json.dumps(record,indent=2)+'\n')
print(json.dumps(record),flush=True)
if result.returncode:print(log.read_text()[-10000:])
raise SystemExit(result.returncode)
