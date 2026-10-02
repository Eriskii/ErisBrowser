function check(v){if(!v)throw new Error('policy prerequisite');}
var n=new Text('A\uD834\uDD1EB'); n.replaceData(2,1,'\uDD1E'); check(n.data==='A\uD834\uDD1EB');
n.replaceData(2,1,'');
