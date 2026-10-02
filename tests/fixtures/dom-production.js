// Independent exact-production oracle. Loading defines cases; invoke exactly
// one domProductionCases[name]() in each fresh Runtime/Document, in each mode.
// Expected completion is true, with all assertions reached. No engine was used
// to select these cases or their literal UTF-16 expectations.
// https://webidl.spec.whatwg.org/#es-DOMString
// https://webidl.spec.whatwg.org/#es-attributes
// https://dom.spec.whatwg.org/#interface-characterdata
// https://dom.spec.whatwg.org/#interface-processinginstruction
var domProductionCases = (function () {
    function check(value, label) { if (!value) throw new Error(label); }
    function units(value, expected) {
        check(typeof value === 'string' && value.length === expected.length, 'UTF16 length');
        for (var i = 0; i < expected.length; i++)
            check(value.charCodeAt(i) === expected[i], 'UTF16 unit ' + i);
    }
    function data(node, expected) {
        units(node.data, expected);
        check(node.length === expected.length, 'CharacterData length');
    }
    function thrown(action) {
        var caught, didThrow = false;
        try { action(); } catch (error) { didThrow = true; caught = error; }
        check(didThrow, 'required abrupt completion');
        return caught;
    }
    function typeError(action) { check(thrown(action) instanceof TypeError, 'TypeError'); }
    function invalid(action) {
        var error = thrown(action);
        check(error instanceof DOMException && error.name === 'InvalidCharacterError' && error.code === 5,
              'InvalidCharacterError');
    }
    function alternate(getter) {
        var target = (function () {}).bind(null);
        Object.defineProperty(target, 'prototype', {get: getter, configurable: true});
        return target;
    }
    return {
        constructors_and_factory_exact: function () {
            var nodes = [new Text('A\uD800B'), new Comment('\uDC00'),
                document.createTextNode('\uD800\uD834\uDD1E'),
                new ProcessingInstruction('xml', '\uD800'),
                document.createProcessingInstruction('p', '\uDC00')];
            var expected = [[65,55296,66],[56320],[55296,55348,56606],[55296],[56320]];
            for (var i = 0; i < nodes.length; i++) {
                data(nodes[i], expected[i]);
                check(nodes[i].parentNode === null, 'new node detached');
            }
            check(nodes[0] instanceof Text && nodes[1] instanceof Comment && nodes[2] instanceof Text,
                  'genuine Text and Comment');
            check(nodes[3] instanceof ProcessingInstruction && nodes[3].target === 'xml' &&
                  nodes[4].target === 'p', 'genuine PI and exact targets');
            return true;
        },
        defaults_and_required_arguments: function () {
            var constructors = [Text, Comment];
            for (var i = 0; i < constructors.length; i++) {
                var C = constructors[i];
                check(new C().data === '' && new C(undefined).data === '', 'optional empty default');
                check(new C(null).data === 'null', 'constructor null converts normally');
                typeError(function () { C('\uD800'); });
            }
            check(new ProcessingInstruction('p').data === '' &&
                  new ProcessingInstruction('p', undefined).data === '', 'PI optional empty default');
            check(new ProcessingInstruction('p', null).data === 'null', 'PI null conversion');
            check(document.createTextNode(undefined).data === 'undefined' &&
                  document.createProcessingInstruction('p', undefined).data === 'undefined',
                  'required factory argument has no default');
            typeError(function () { document.createTextNode(); });
            typeError(function () { document.createProcessingInstruction('p'); });
            typeError(function () { new ProcessingInstruction(); });
            typeError(function () { ProcessingInstruction('p', '\uD800'); });
            return true;
        },
        setter_defaults_and_exact: function () {
            var d = Object.getOwnPropertyDescriptor(CharacterData.prototype, 'data');
            check(typeof d.get === 'function' && typeof d.set === 'function', 'saved real accessors');
            var nodes = [new Text(''),new Comment(''),new ProcessingInstruction('p')];
            for (var i = 0; i < nodes.length; i++) {
                var n = nodes[i];
                check(d.set.call(n, '\uD800') === undefined, 'setter result'); data(n,[55296]);
                d.set.call(n, null); data(n,[]);
                d.set.call(n, undefined); data(n,[117,110,100,101,102,105,110,101,100]);
                d.set.call(n); data(n,[117,110,100,101,102,105,110,101,100]);
                d.set.call(n, {toString:function(){return null;}}); data(n,[110,117,108,108]);
                d.set.call(n, '\uDC00'); units(d.get.call(n),[56320]);
            }
            return true;
        },
        authentic_brands_precede_conversion: function () {
            var n = new Text('x'), set = Object.getOwnPropertyDescriptor(CharacterData.prototype,'data').set;
            set.call(n,'\uD800'); data(n,[55296]);
            var calls = 0, argument = {valueOf:function(){calls++;return 0;},
                                      toString:function(){calls++;return '\uDC00';}};
            var fake = Object.create(CharacterData.prototype);
            typeError(function(){set.call(fake,argument);});
            var methods = ['substringData','appendData','insertData','deleteData','replaceData'];
            for (var i=0;i<methods.length;i++) {
                var operation = CharacterData.prototype[methods[i]];
                check(typeof operation === 'function', 'operation present');
                typeError(function(){operation.call(fake,argument,argument,argument);});
            }
            var makeText=document.createTextNode, makePI=document.createProcessingInstruction;
            check(makeText.call(document,'x').data==='x' && makePI.call(document,'p','x').data==='x',
                  'successful authentic factory calls');
            typeError(function(){makeText.call({},argument);});
            typeError(function(){makePI.call({},argument,argument);});
            check(calls===0,'brand failure must not coerce');
            typeError(function(){set.call(n,Symbol('bad'));}); data(n,[55296]);
            return true;
        },
        units_substring_and_bounds: function () {
            var n=new Text('A\uD800B\uDC00');
            units(n.substringData(1,2),[55296,66]);
            units(n.substringData(3,100),[56320]);
            units(n.substringData(4,0),[]);
            units(n.substringData(4294967297,1),[55296]);
            var error=thrown(function(){n.substringData(5,0);});
            check(error instanceof DOMException && error.name==='IndexSizeError' && error.code===1,
                  'range after exact unit length');
            data(n,[65,55296,66,56320]);
            return true;
        },
        complete_splice_repairs: function () {
            var prefix=new Text('A\uD800'); prefix.appendData('\uDC00'); data(prefix,[65,55296,56320]);
            var suffix=new Text('\uDC00B'); suffix.insertData(0,'\uD800'); data(suffix,[55296,56320,66]);
            var deletion=new Text('A\uD800x\uDC00B'); deletion.deleteData(2,1); data(deletion,[65,55296,56320,66]);
            var both=new Text('\uD800\uDC00'); both.insertData(1,'\uDC00\uD800'); data(both,[55296,56320,55296,56320]);
            return true;
        },
        splice_retains_other_isolated_units: function () {
            var n=new Text('\uD800A\uDC00'); n.replaceData(1,1,'B'); data(n,[55296,66,56320]);
            check(n.deleteData(1,0)===undefined,'zero deletion result'); data(n,[55296,66,56320]);
            n.replaceData(1,100,'\uDC00'); data(n,[55296,56320]);
            n.deleteData(0,2); data(n,[]);
            n.appendData('\uD800'); data(n,[55296]);
            return true;
        },
        conversions_use_fresh_data: function () {
            var n=new Text('old'), trace='';
            n.replaceData(
                {valueOf:function(){trace+='O';n.data='old offset';return 1;}},
                {valueOf:function(){trace+='C';n.data='old count';return 1;}},
                {toString:function(){trace+='D';n.data='\uD800A\uDC00';return 'B';}});
            check(trace==='OCD','offset count data order'); data(n,[55296,66,56320]);
            n.appendData({toString:function(){n.data='\uD800';return '\uDC00';}});
            data(n,[55296,56320]);
            var token={}, caught=thrown(function(){n.insertData(0,{toString:function(){
                n.data='\uDC00';throw token;
            }});});
            check(caught===token,'abrupt identity'); data(n,[56320]);
            return true;
        },
        constructor_conversion_before_prototype: function () {
            var constructors=[Text,Comment];
            for(var i=0;i<constructors.length;i++) {
                var C=constructors[i],trace='',custom=Object.create(C.prototype);
                var target=alternate(function(){trace+='P';return custom;});
                var node=Reflect.construct(C,[{toString:function(){trace+='D';return '\uD800';}}],target);
                check(trace==='DP'&&Object.getPrototypeOf(node)===custom,'converted data then prototype');
                data(node,[55296]);
            }
            var order='',piCustom=Object.create(ProcessingInstruction.prototype);
            var piTarget=alternate(function(){order+='P';return piCustom;});
            var pi=Reflect.construct(ProcessingInstruction,[
                {toString:function(){order+='T';return 'p';}},
                {toString:function(){order+='D';return '\uDC00';}}],piTarget);
            check(order==='TDP'&&Object.getPrototypeOf(pi)===piCustom,'PI conversions then prototype');
            data(pi,[56320]);
            return true;
        },
        constructor_abrupt_preserves_callback_prefix: function () {
            var held=new Text('old'),token={},trace='';
            var target=alternate(function(){trace+='P';held.data='\uDC00';throw token;});
            var error=thrown(function(){Reflect.construct(Text,[{toString:function(){
                trace+='D';held.data='\uD800';return '\uD800';
            }}],target);});
            check(error===token&&trace==='DP','getter abrupt after exact conversion'); data(held,[56320]);
            trace='';
            var piTarget=alternate(function(){trace+='P';throw token;});
            error=thrown(function(){Reflect.construct(ProcessingInstruction,[
                {toString:function(){trace+='T';return 'invalid name';}},
                {toString:function(){trace+='D';return '?>\uD800';}}],piTarget);});
            check(error===token&&trace==='TDP','prototype throw precedes PI validation');
            return true;
        },
        data_json_and_clone_roundtrips: function () {
            var value='\uD800A\uDC00';
            check(JSON.stringify(value)==='"\\ud800A\\udc00"','well-formed JSON escaping');
            units(JSON.parse(JSON.stringify(value)),[55296,65,56320]);
            var parent=document.createElement('div');
            parent.appendChild(new Text(value));parent.appendChild(new Comment('\uDC00'));
            parent.appendChild(new ProcessingInstruction('p','\uD800'));
            var clone=parent.cloneNode(true),originals=parent.childNodes,copies=clone.childNodes;
            check(clone!==parent&&copies.length===3,'deep clone shape');
            var expected=[[55296,65,56320],[56320],[55296]];
            for(var i=0;i<3;i++) {
                check(copies[i]!==originals[i]&&copies[i].nodeType===originals[i].nodeType,'new authentic clone');
                data(copies[i],expected[i]);units(JSON.parse(JSON.stringify(copies[i].data)),expected[i]);
            }
            originals[0].data='changed';data(copies[0],[55296,65,56320]);
            check(copies[2].target==='p','PI clone target');
            return true;
        },
        pi_validation_creation_versus_mutation: function () {
            var a=new ProcessingInstruction('xml','?\uD800>');data(a,[63,55296,62]);
            var b=document.createProcessingInstruction('p','?\uDC00>');data(b,[63,56320,62]);
            invalid(function(){new ProcessingInstruction('p','?>\uD800');});
            invalid(function(){document.createProcessingInstruction('p','\uDC00?>');});
            invalid(function(){new ProcessingInstruction('\uD800','');});
            a.data='?>\uD800';data(a,[63,62,55296]);
            var clone=a.cloneNode(false);data(clone,[63,62,55296]);check(clone.target==='xml','clone keeps target');
            b.data='?';b.appendData('>\uDC00');data(b,[63,62,56320]);
            b.replaceData(0,2,'?>');data(b,[63,62,56320]);
            return true;
        }
    };
}());
