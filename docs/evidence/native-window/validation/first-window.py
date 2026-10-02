from pathlib import Path
import json,sys
root=Path('/home/eriskii/Documents/Programming/Projects/ErisBrowser')
sys.path.insert(0,str(root/'tools'));from run import launch_environment
sys.path.insert(0,str(root/'tools/vulkan-raster-probe'));from run_browser_host import run_supervised
out=Path('/tmp/eris-native-window-1/first-window');out.mkdir()
env=launch_environment();env['LD_LIBRARY_PATH']='/nix/store/iahmf8kl215rjcvb3zz3l7i9w2sf9x5v-vulkan-loader-1.4.357.0/lib:'+env.get('LD_LIBRARY_PATH','')
record,stdout=run_supervised(['/tmp/eris-native-window-1/eris-browser-native',str(root/'tools/native-raster-fixtures/admitted.html'),'--presenter=vulkan','--raster=gpu','--vulkan-verify-frames','1','--exit-after','6'],out,'admitted-initial',env,18)
print(json.dumps(record))
print((out/'admitted-initial.stderr.log').read_text()[-6000:])
