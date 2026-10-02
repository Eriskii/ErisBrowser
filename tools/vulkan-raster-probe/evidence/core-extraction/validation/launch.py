import os,sys
from pathlib import Path
sys.path.insert(0,"/home/eriskii/Documents/Programming/Projects/ErisBrowser/tools")
from run import launch_environment
os.execve(sys.executable,[sys.executable,*sys.argv[1:]],launch_environment())
