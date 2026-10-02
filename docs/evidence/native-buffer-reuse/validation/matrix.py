import subprocess,sys
from pathlib import Path
out=Path(__file__).resolve().parent
checks=[
 ('default-clippy-198-1','1.98.0','Cargo.toml','-','clippy','--all-targets','--','-D','warnings'),
 ('core-clippy-198-1','1.98.0','crates/raster-core/Cargo.toml','gpu','clippy','--all-targets','--','-D','warnings'),
 ('core-tests-198-1','1.98.0','crates/raster-core/Cargo.toml','gpu','test'),
 ('probe-clippy-198-1','1.98.0','tools/vulkan-raster-probe/Cargo.toml','browser-bridge','clippy','--all-targets','--','-D','warnings'),
 ('probe-tests-198-1','1.98.0','tools/vulkan-raster-probe/Cargo.toml','browser-bridge','test'),
 ('probe-clippy-188-3','1.88.0','tools/vulkan-raster-probe/Cargo.toml','browser-bridge','clippy','--all-targets','--','-D','warnings'),
 ('probe-tests-188-3','1.88.0','tools/vulkan-raster-probe/Cargo.toml','browser-bridge','test'),
]
for args in checks:
 result=subprocess.run([sys.executable,str(out/'check.py'),*args],close_fds=True)
 if result.returncode: raise SystemExit(result.returncode)
