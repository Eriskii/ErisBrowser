import hashlib,json,os,pathlib,subprocess,sys,time
root=pathlib.Path('/home/eriskii/Documents/Programming/Projects/ErisBrowser');out=pathlib.Path(__file__).parent;version=sys.argv[1]
env=os.environ.copy();env.update(CARGO_HOME='/tmp/eris-vulkan-browser-cargo-home',RUSTUP_HOME='/tmp/eris-vulkan-rustup',CARGO_TARGET_DIR='/tmp/eris-vulkan-raster-prototype/target-'+('msrv' if version=='1.88.0' else '198'),CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER='python3 tools/ci_test_runner.py')
probe='tools/vulkan-raster-probe/Cargo.toml';cargo=['cargo','+'+version]
checks=[('root-fmt',['fmt','--all','--','--check']),('probe-fmt',['fmt','--manifest-path',probe,'--','--check'])]
for suffix,feature in [('bridge','raster-bridge'),('combined','raster-bridge,vulkan-presenter')]:
 checks += [('root-clippy-'+suffix,['clippy','--locked','--offline','--all-targets','--features',feature,'--','-D','warnings']),('root-test-'+suffix,['test','--locked','--offline','--lib','--features',feature,'graphics::raster_bridge::','--','--include-ignored'])]
checks += [('probe-clippy',['clippy','--locked','--offline','--manifest-path',probe,'--all-targets','--features','browser-bridge','--','-D','warnings']),('probe-test',['test','--locked','--offline','--manifest-path',probe,'--features','browser-bridge']),('probe-build',['build','--locked','--offline','--manifest-path',probe,'--features','browser-bridge'])]
records=[]
for name,args in checks:
 path=out/(version+'-'+name+'.log');start=time.monotonic()
 # Cargo resolves a relative runner from each package workspace root.
 checkenv=env.copy();checkenv['CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER']='python3 '+str(root/'tools/ci_test_runner.py')
 with path.open('xb') as log:run=subprocess.run(cargo+args,cwd=root,env=checkenv,stdout=log,stderr=subprocess.STDOUT)
 text=path.read_text();result_ok=run.returncode==0
 if name.startswith('root-test-'):result_ok &= 'test result: ok. 28 passed; 0 failed; 0 ignored;' in text
 rec={'name':name,'command':cargo+args,'exit':run.returncode,'expected_count_matched':result_ok,'seconds':time.monotonic()-start,'log':path.name,'sha256':hashlib.sha256(path.read_bytes()).hexdigest()};records.append(rec);(out/(version+'-checks.json')).write_text(json.dumps(records,indent=2)+'\n');print(version,name,run.returncode,result_ok,flush=True)
 if not result_ok:sys.exit(run.returncode or 1)
