import json,pathlib,subprocess,sys
out=pathlib.Path(__file__).parent;root=pathlib.Path('/home/eriskii/Documents/Programming/Projects/ErisBrowser')
loader='/nix/store/iahmf8kl215rjcvb3zz3l7i9w2sf9x5v-vulkan-loader-1.4.357.0/lib'
browser='/tmp/eris-vulkan-worker-text-2/eris-browser-stripped'
common=['--allow-experimental-gpu','--loader-directory',loader]
tasks=[('glyph-host-1','run_glyph_host.py',['--binary',str(out/'eris-vulkan-glyph-check-1.98.0'),'--baselines',str(root/'tools/vulkan-raster-probe/glyph-fixtures/parent-baselines'),'--output',str(out/'glyph-host-1')]),('browser-host-1','run_browser_host.py',['--binary',str(out/'eris-vulkan-browser-check-1.98.0'),'--browser',browser,'--output-dir',str(out/'browser-host-1')]),('worker-text-host-1','run_worker_text_host.py',['--binary',str(out/'eris-vulkan-worker-text-check-1.98.0'),'--browser',browser,'--output',str(out/'worker-text-host-1')])]
records=[]
for name,runner,args in tasks:
 command=[sys.executable,str(out/'launch.py'),str(root/'tools/vulkan-raster-probe'/runner),*args,*common]
 with (out/(name+'-launcher.log')).open('xb') as log:result=subprocess.run(command,cwd=root,stdout=log,stderr=subprocess.STDOUT)
 summary=json.loads((out/name/'host-results.json').read_text()) if (out/name/'host-results.json').exists() else {}
 records.append({'name':name,'command':command,'exit':result.returncode,'success':summary.get('success',False),'adapter_count':summary.get('adapter_count',0)})
 (out/'gpu-runs.json').write_text(json.dumps(records,indent=2)+'\n')
 print(name,result.returncode,summary.get('success'),summary.get('adapter_count'),flush=True)
 if result.returncode or not summary.get('success'):sys.exit(1)
