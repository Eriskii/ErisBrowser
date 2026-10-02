from pathlib import Path
import subprocess,sys
here=Path(__file__).resolve().parent
checks=[
 ('native-tests-1880-host','1.88.0','vulkan-raster','test',['--','--include-ignored']),
 ('default-clippy-1880','1.88.0','-','clippy',['--all-targets','--','-D','warnings']),
 ('presenter-clippy-1880','1.88.0','vulkan-presenter','clippy',['--all-targets','--','-D','warnings']),
 ('presenter-bin-tests-1880','1.88.0','vulkan-presenter','test',['--bin','eris-browser','--','--include-ignored']),
 ('default-bin-tests-1880','1.88.0','-','test',['--bin','eris-browser','--','--include-ignored']),
 ('native-clippy-1980','1.98.0','vulkan-raster','clippy',['--all-targets','--','-D','warnings']),
 ('native-tests-1980-final','1.98.0','vulkan-raster','test',['--','--include-ignored']),
 ('native-release-1980','1.98.0','vulkan-raster','build',['--release','--bin','eris-browser']),
]
for label,tc,features,action,tail in checks:
 subprocess.run([sys.executable,str(here/'check.py'),label,tc,'Cargo.toml',features,action,*tail],check=True)
