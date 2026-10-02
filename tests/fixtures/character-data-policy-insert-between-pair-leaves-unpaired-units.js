function check(v){if(!v)throw new Error('policy prerequisite');}
var n=new Text('A\uD834\uDD1EB'); n.insertData(0,''); check(n.data==='A\uD834\uDD1EB');
n.insertData(2,'x');
