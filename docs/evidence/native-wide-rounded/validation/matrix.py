from pathlib import Path
import subprocess,sys
here=Path(__file__).resolve().parent
toolchain=sys.argv[1]; suffix=toolchain.replace('.','')
checks=[
 ('core-default','crates/raster-core/Cargo.toml','-','test',[]),
 ('core-gpu','crates/raster-core/Cargo.toml','gpu','test',[]),
 ('core-clippy-default','crates/raster-core/Cargo.toml','-','clippy',['--all-targets','--','-D','warnings']),
 ('core-clippy-gpu','crates/raster-core/Cargo.toml','gpu','clippy',['--all-targets','--','-D','warnings']),
 ('bridge-tests','Cargo.toml','raster-bridge','test',['--lib','graphics::raster_bridge::','--','--include-ignored']),
 ('bridge-clippy','Cargo.toml','raster-bridge','clippy',['--all-targets','--','-D','warnings']),
 ('native-clippy','Cargo.toml','vulkan-raster','clippy',['--all-targets','--','-D','warnings']),
 ('probe-tests','tools/vulkan-raster-probe/Cargo.toml','browser-bridge','test',[]),
]
for label,manifest,features,action,tail in checks:
 subprocess.run([sys.executable,str(here/'check.py'),label+'-'+suffix,toolchain,manifest,features,action,*tail],check=True)
