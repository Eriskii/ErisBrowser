import subprocess,sys
from pathlib import Path
p=Path(__file__).resolve().parent
toolchain=sys.argv[1]; short=toolchain.replace('.','')
tasks=[
('core-clippy-default','crates/raster-core/Cargo.toml','-','clippy',['--all-targets','--','-D','warnings']),
('core-clippy-gpu','crates/raster-core/Cargo.toml','gpu','clippy',['--all-targets','--','-D','warnings']),
('core-default','crates/raster-core/Cargo.toml','-','test',[]),
('core-gpu','crates/raster-core/Cargo.toml','gpu','test',[]),
('native-clippy-final','Cargo.toml','vulkan-raster','clippy',['--all-targets','--','-D','warnings']),
('native-bin','Cargo.toml','vulkan-raster','test',['--bin','eris-browser','--','--include-ignored']),
('default-clippy','Cargo.toml','-','clippy',['--all-targets','--','-D','warnings']),
('default-bin','Cargo.toml','-','test',['--bin','eris-browser','--','--include-ignored']),
('presenter-clippy','Cargo.toml','vulkan-presenter','clippy',['--all-targets','--','-D','warnings']),
('presenter-bin','Cargo.toml','vulkan-presenter','test',['--bin','eris-browser','--','--include-ignored']),
('bridge-clippy','Cargo.toml','raster-bridge','clippy',['--all-targets','--','-D','warnings']),
('bridge-tests','Cargo.toml','raster-bridge','test',['--lib','graphics::raster_bridge::','--','--include-ignored']),
('probe-clippy','tools/vulkan-raster-probe/Cargo.toml','browser-bridge','clippy',['--all-targets','--','-D','warnings']),
('probe-tests','tools/vulkan-raster-probe/Cargo.toml','browser-bridge','test',[]),
]
for label,manifest,features,action,tail in tasks:
 result=subprocess.run([sys.executable,str(p/'check.py'),label+'-'+short,toolchain,manifest,features,action,*tail])
 if result.returncode:raise SystemExit(result.returncode)
