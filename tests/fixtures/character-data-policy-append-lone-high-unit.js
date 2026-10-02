function check(v){if(!v)throw new Error('policy prerequisite');}
var n=new Text('old'); n.appendData(''); check(n.data==='old');
n.appendData('\uD800');
