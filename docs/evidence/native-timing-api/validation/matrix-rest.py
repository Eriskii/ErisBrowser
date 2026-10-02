from pathlib import Path
import subprocess,sys
here=Path(__file__).resolve().parent
checks=[
 ('native-clippy-1880-report-fix','1.88.0','clippy',['--all-targets','--','-D','warnings']),
 ('native-bin-tests-1880-report-fix','1.88.0','test',['--bin','eris-browser','--','--include-ignored']),
 ('native-clippy-1980-report-fix','1.98.0','clippy',['--all-targets','--','-D','warnings']),
 ('native-tests-1980-final','1.98.0','test',['--','--include-ignored']),
 ('native-release-1980','1.98.0','build',['--release','--bin','eris-browser']),
]
for label,tc,action,tail in checks:
 subprocess.run([sys.executable,str(here/'check.py'),label,tc,'Cargo.toml','vulkan-raster',action,*tail],check=True)
