function check(v){if(!v)throw new Error('policy prerequisite');}
var n=new Text('A\uD834\uDD1EB'); check(n.substringData(1,2)==='\uD834\uDD1E'); n.deleteData(2,0); check(n.data==='A\uD834\uDD1EB');
n.deleteData(1,1);
