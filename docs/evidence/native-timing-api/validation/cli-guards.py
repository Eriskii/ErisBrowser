import hashlib,json,os,subprocess,sys
from pathlib import Path
binary=Path(sys.argv[1]).resolve();out=Path(__file__).resolve().parent
cases=[
 ('one',['--benchmark-native','1']),('too-many',['--benchmark-native','129']),
 ('negative',['--benchmark-native','-1']),('missing',['--benchmark-native']),
 ('software',['--benchmark-native','2']),('headless',['--benchmark-native','2','--render','--presenter=vulkan']),
 ('other-benchmark',['--benchmark-native','2','--benchmark','2','--presenter=vulkan']),
 ('worker-benchmark',['--benchmark-native','2','--benchmark-worker','2','--presenter=vulkan']),
 ('verification',['--benchmark-native','2','--vulkan-verify-frames','1','--presenter=vulkan']),
 ('check-and-measure',['--benchmark-native','2','--benchmark-native-check','--presenter=vulkan']),
 ('check-two-first',['--vulkan-verify-frames','2','--benchmark-native-check','--presenter=vulkan']),
 ('check-two-last',['--benchmark-native-check','--vulkan-verify-frames','2','--presenter=vulkan']),
 ('check-headless',['--benchmark-native-check','--render','--presenter=vulkan']),
 ('screenshot',['--benchmark-native','2','--window-screenshot','/tmp/eris-native-timing-forbidden.png','--presenter=vulkan']),
 ('check-screenshot',['--benchmark-native-check','--window-screenshot','/tmp/eris-native-timing-forbidden.png','--presenter=vulkan']),
]
rows=[]
env=os.environ.copy()
for key in ('DISPLAY','WAYLAND_DISPLAY','XDG_RUNTIME_DIR'):env.pop(key,None)
for name,args in cases:
 result=subprocess.run([str(binary),*args],env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=5,close_fds=True)
 row={'name':name,'args':args,'exit':result.returncode,'stdout':result.stdout.decode(),'stderr':result.stderr.decode()}
 row['passed']=result.returncode!=0 and not result.stdout and len(result.stderr)<4096 and b'Unable to open browser window' not in result.stderr
 rows.append(row)
record={'schema':1,'binary':str(binary),'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'cases':rows,'success':all(r['passed'] for r in rows)}
p=out/'cli-guards.json';p.write_text(json.dumps(record,indent=2)+'\n')
print(json.dumps({'passed':sum(r['passed'] for r in rows),'total':len(rows),'record':str(p)}))
raise SystemExit(0 if record['success'] else 1)
