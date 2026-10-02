import hashlib,json,subprocess,sys,time
from pathlib import Path
out=Path(__file__).resolve().parent
label,directory=sys.argv[1:]
log=out/(label+'.log')
argv=[sys.executable,'-m','unittest','discover','-s',directory,'-p','test_*.py']
start=time.time()
with log.open('xb') as stream:
 result=subprocess.run(argv,stdout=stream,stderr=subprocess.STDOUT,close_fds=True)
record=dict(command=argv,exit=result.returncode,elapsed_seconds=time.time()-start,log=log.name,sha256=hashlib.sha256(log.read_bytes()).hexdigest())
(out/(label+'.json')).write_text(json.dumps(record,indent=2)+'\n')
print(json.dumps(record),flush=True)
if result.returncode: print(log.read_text()[-5000:])
raise SystemExit(result.returncode)
