import hashlib,json,os,pathlib,subprocess,sys,time
root=pathlib.Path('/home/eriskii/Documents/Programming/Projects/ErisBrowser');out=pathlib.Path(__file__).parent;version=sys.argv[1]
env=os.environ.copy();env.update(CARGO_HOME='/tmp/eris-vulkan-browser-cargo-home',RUSTUP_HOME='/tmp/eris-vulkan-rustup',CARGO_TARGET_DIR='/tmp/eris-vulkan-raster-prototype/target-'+('msrv' if version=='1.88.0' else '198'))
core='crates/raster-core/Cargo.toml';probe='tools/vulkan-raster-probe/Cargo.toml';cargo=['cargo','+'+version]
checks=[('core-fmt',['fmt','--manifest-path',core,'--','--check']),('core-clippy-gpu',['clippy','--locked','--offline','--manifest-path',core,'--all-targets','--features','gpu','--','-D','warnings']),('core-test-gpu',['test','--locked','--offline','--manifest-path',core,'--features','gpu']),('probe-fmt',['fmt','--manifest-path',probe,'--','--check']),('probe-clippy-default',['clippy','--locked','--offline','--manifest-path',probe,'--all-targets','--','-D','warnings']),('probe-test-default',['test','--locked','--offline','--manifest-path',probe]),('probe-clippy-feature',['clippy','--locked','--offline','--manifest-path',probe,'--all-targets','--features','browser-bridge','--','-D','warnings']),('probe-test-feature',['test','--locked','--offline','--manifest-path',probe,'--features','browser-bridge']),('probe-build-feature',['build','--locked','--offline','--manifest-path',probe,'--features','browser-bridge'])]
records=[]
for name,args in checks:
 path=out/(version+'-'+name+'.log');start=time.monotonic()
 with path.open('xb') as log:run=subprocess.run(cargo+args,cwd=root,env=env,stdout=log,stderr=subprocess.STDOUT)
 rec={'name':name,'command':cargo+args,'exit':run.returncode,'seconds':time.monotonic()-start,'log':path.name,'sha256':hashlib.sha256(path.read_bytes()).hexdigest()};records.append(rec);(out/(version+'-checks.json')).write_text(json.dumps(records,indent=2)+'\n');print(version,name,run.returncode,flush=True)
 if run.returncode:sys.exit(run.returncode)
