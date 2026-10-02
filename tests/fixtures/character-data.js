// Independent preparation oracle; no cases run by loading this file.
// Run one named case in a fresh Runtime/Document after the PI/accessor increment.
// DOM: https://dom.spec.whatwg.org/#interface-characterdata
// Web IDL: https://webidl.spec.whatwg.org/#es-unsigned-long
var characterDataCases = (function () {
    function check(value, message) { if (!value) throw new Error(message); }
    function method(name) {
        var f = CharacterData.prototype[name];
        check(typeof f === 'function', name + ' callable prerequisite');
        return f;
    }
    function typeError(action) {
        var caught;
        try { action(); } catch (error) { caught = error; }
        check(caught instanceof TypeError, 'TypeError');
    }
    function indexError(action) {
        var caught;
        try { action(); } catch (error) { caught = error; }
        check(caught instanceof DOMException && caught.name === 'IndexSizeError' && caught.code === 1,
              'IndexSizeError identity');
    }
    return {
        metadata: function () {
            var names = ['substringData','appendData','insertData','deleteData','replaceData'];
            var lengths = [2,1,2,2,3], a = new Text('a'), b = new Comment('b');
            for (var i = 0; i < names.length; i++) {
                var name = names[i], f = method(name);
                var d = Object.getOwnPropertyDescriptor(CharacterData.prototype, name);
                check(d.value === f && d.writable && d.enumerable && d.configurable, 'method flags');
                check(f.name === name && f.length === lengths[i], 'name/length');
                check(a[name] === f && b[name] === f, 'shared inherited identity');
                check(Object.getOwnPropertyDescriptor(a,name) === undefined, 'not an own copy');
                check(Object.getOwnPropertyDescriptor(f,'prototype') === undefined, 'no constructor prototype');
                typeError(function () { new f(); });
            }
            return true;
        },
        scalar_family_and_tree: function () {
            var nodes = [new Text('ab'),new Comment('ab'),new ProcessingInstruction('probe','ab')];
            var holder = document.createElement('div');
            for (var i = 0; i < nodes.length; i++) {
                var n = nodes[i]; holder.appendChild(n);
                check(n.appendData('c') === undefined && n.data === 'abc', 'append');
                check(n.insertData(1,'\u00e9') === undefined && n.data === 'a\u00e9bc', 'insert');
                check(n.deleteData(2,1) === undefined && n.data === 'a\u00e9c', 'delete');
                check(n.replaceData(1,1,'\uD834\uDD1E') === undefined && n.data === 'a\uD834\uDD1Ec', 'replace');
                check(n.length === 4 && n.substringData(1,2) === '\uD834\uDD1E', 'UTF16 indexing');
                check(n.parentNode === holder && holder.childNodes[i] === n, 'same node and tree position');
            }
            check(nodes[2].target === 'probe', 'PI target unchanged');
            return true;
        },
        substring_half_pairs_and_bounds: function () {
            var n = new Text('A\uD834\uDD1EB');
            check(n.substringData(1,1) === '\uD834', 'high unit result');
            check(n.substringData(2,1) === '\uDD1E', 'low unit result');
            check(n.substringData(3,99) === 'B' && n.substringData(4,99) === '', 'clipped count/end');
            indexError(function () { n.substringData(5,0); });
            check(n.data === 'A\uD834\uDD1EB', 'substring never mutates');
            return true;
        },
        unsigned_long_modulo_and_defaults: function () {
            var n = new Text('abcd');
            var zero = [undefined,null,false,NaN,Infinity,-Infinity,-0,-0.5,4294967296];
            for (var i = 0; i < zero.length; i++) check(n.substringData(zero[i],1) === 'a','zero conversion');
            check(n.substringData(4294967297,1) === 'b','positive wrap');
            check(n.substringData(-4294967295,1) === 'b','negative wrap');
            check(n.substringData(1.9,1.9) === 'b','truncate');
            check(n.substringData(1,-1) === 'bcd','wrapped count clips');
            check(n.substringData(0,4294967298) === 'ab','count wraps');
            check(n.substringData(0,Infinity) === '','infinite count zero');
            indexError(function () { n.substringData(-1,0); });
            typeError(function () { n.substringData(Symbol('offset'),1); });
            typeError(function () { n.substringData(0,Symbol('count')); });
            return true;
        },
        brands_and_arity_precede_conversion: function () {
            var n = new Text('ab'), calls = 0;
            var v = {valueOf:function(){calls++;return 0;},toString:function(){calls++;return 'x';}};
            var names = ['substringData','appendData','insertData','deleteData','replaceData'];
            var good = [[0,0],[''],[0,''],[0,0],[0,0,'']];
            var short = [[v],[],[v],[v],[v,v]];
            for (var i=0; i<names.length; i++) {
                var f=method(names[i]); f.apply(n,good[i]);
                typeError(function(){f.apply(n,short[i]);});
                typeError(function(){f.call(Object.create(CharacterData.prototype),v,v,v);});
                typeError(function(){f.call(document,v,v,v);});
            }
            check(calls===0 && n.data==='ab','no conversion before arity/brand refusal');
            return true;
        },
        replace_conversion_order_and_fresh_data: function () {
            var n=new Text('old'),trace='';
            var o={valueOf:function(){trace+='O';n.data='abcd';return 1;}};
            var c={valueOf:function(){check(n.data==='abcd','after offset');trace+='C';n.data='uvwxyz';return 2;}};
            var d={}; d[Symbol.toPrimitive]=function(hint){check(hint==='string'&&n.data==='uvwxyz','data conversion');trace+='D';n.data='WXYZ';return '!';};
            check(n.replaceData(o,c,d)===undefined,'return');
            check(trace==='OCD'&&n.data==='W!Z','splice latest data');
            return true;
        },
        callback_growth_and_shrink_change_bounds: function () {
            var n=new Text('a'),trace='';
            n.replaceData(4,0,{toString:function(){trace+='G';n.data='abcdef';return '!';}});
            check(n.data==='abcd!ef'&&trace==='G','growth rescues offset');
            indexError(function(){n.insertData(2,{toString:function(){trace+='S';n.data='x';return '!';}});});
            check(trace==='GS'&&n.data==='x','shrink prefix retained; no outer insertion');
            return true;
        },
        append_uses_length_after_data_conversion: function () {
            var n=new Text('old'),trace='';
            n.appendData({toString:function(){trace+='D';n.data='newer';return '!';}});
            check(trace==='D'&&n.data==='newer!','fresh append offset');
            return true;
        },
        substring_and_delete_read_after_both_numbers: function () {
            var n=new Text('old'),trace='';
            var o={valueOf:function(){trace+='O';n.data='abcdef';return 1;}};
            var c={valueOf:function(){trace+='C';n.data='WXYZ';return 2;}};
            check(n.substringData(o,c)==='XY'&&trace==='OC'&&n.data==='WXYZ','substring snapshot after conversion');
            trace=''; check(n.deleteData(o,c)===undefined&&trace==='OC'&&n.data==='WZ','delete current data');
            return true;
        },
        abrupt_identity_precedes_range_check: function () {
            var n=new Text('abc'),marker={},caught,trace='';
            n.replaceData(0,0,'');
            try { n.replaceData(99,0,{toString:function(){trace+='D';n.data='prefix';throw marker;}}); } catch(e){caught=e;}
            check(caught===marker&&trace==='D'&&n.data==='prefix','data abrupt beats invalid offset');
            trace='';caught=undefined;
            try { n.replaceData({valueOf:function(){trace+='O';throw marker;}},{valueOf:function(){trace+='C';return 0;}},{toString:function(){trace+='D';return '';}}); } catch(e){caught=e;}
            check(caught===marker&&trace==='O','offset abrupt stops later conversions');
            typeError(function(){n.replaceData(0,Symbol('count'),{toString:function(){trace+='D';return '';}});});
            check(trace==='O','count abrupt stops data');
            return true;
        },
        domstring_null_and_extra_arguments: function () {
            var n=new Text(''),unused={toString:function(){throw new Error('extra coerced');}};
            n.appendData(null,unused);check(n.data==='null','method null is ordinary DOMString');
            n.insertData(0,undefined,unused);check(n.data==='undefinednull','explicit undefined');
            n.replaceData(0,-1,null,unused);check(n.data==='null','replace null');
            check(n.substringData(0,2,unused)==='nu','extra substring argument ignored');
            typeError(function(){n.appendData(Symbol('data'));});
            check(n.data==='null','symbol failure no outer mutation');
            return true;
        },
        complete_splice_repairs_surrogate_boundaries: function () {
            var n=new Text('A\uD834\uDD1EB');
            n.replaceData(1,1,'\uD834');check(n.data==='A\uD834\uDD1EB','replacement high repairs suffix');
            n.replaceData(2,1,'\uDD1E');check(n.data==='A\uD834\uDD1EB','replacement low repairs prefix');
            n.deleteData(2,0);check(n.data==='A\uD834\uDD1EB','zero deletion inside pair');
            n.insertData(2,'\uDD1E\uD834');
            check(n.data==='A\uD834\uDD1E\uD834\uDD1EB'&&n.length===6,'both insertion seams repair');
            return true;
        },
        pi_mutation_does_not_repeat_creation_validation: function () {
            var n=new ProcessingInstruction('probe','x');
            n.appendData('?>');n.insertData(0,'\u0000');
            check(n.data==='\u0000x?>'&&n.target==='probe','literal PI data retained');
            n.replaceData(1,1,'--');check(n.data==='\u0000--?>','no PI target/data creation validation');
            return true;
        }
    };
})();
