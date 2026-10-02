import hashlib,json,os,subprocess,sys,time
from pathlib import Path
sys.path.insert(0,str(Path.cwd()/'tools'))
from run import launch_environment
out=Path(sys.argv[1]);assert not out.exists()
env=launch_environment()
controller=Path('/nix/store/3swgkdab65v5m97k2x49b9hfgda2v2ib-hyprland-0.56.2/bin/hyprctl')
record={'schema':1,'time_unix':time.time(),'kernel':os.uname().release,'architecture':os.uname().machine,'scope':'Read-only before/after environment snapshot; clocks and power were not locked; monitor settings are not a compositor latency measurement'}
for name in ['scaling_governor','energy_performance_preference']:
 p=Path('/sys/devices/system/cpu/cpu0/cpufreq')/name
 record[name]=p.read_text().strip() if p.is_file() else None
result=subprocess.run([str(controller),'-j','monitors'],env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=3,close_fds=True)
if result.returncode==0 and len(result.stdout)<=262144 and not result.stderr:
 monitors=json.loads(result.stdout);assert type(monitors)is list and len(monitors)<=32
 keys=['id','width','height','refreshRate','scale','dpmsStatus','disabled','vrr']
 record['monitors']=[{k:m.get(k) for k in keys} for m in monitors]
else:record['monitor_query_error']={'exit':result.returncode,'stderr':result.stderr[:4096].decode(errors='replace')}
cmd=['nvidia-smi','--query-gpu=name,driver_version,pstate,power.draw,power.limit,temperature.gpu,clocks.gr,clocks.mem','--format=csv']
result=subprocess.run(cmd,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=5,close_fds=True)
record['gpu_query']={'command':cmd,'exit':result.returncode,'stdout':result.stdout[:8192].decode(errors='replace'),'stderr':result.stderr[:4096].decode(errors='replace')}
out.write_text(json.dumps(record,indent=2)+'\n')
print(json.dumps({'path':str(out),'sha256':hashlib.sha256(out.read_bytes()).hexdigest(),'monitors':len(record.get('monitors',[]))}))
