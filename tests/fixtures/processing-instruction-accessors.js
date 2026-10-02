// Independent accessor oracle. Execute one named case per fresh Runtime/Document.
// Loading this file defines piAccessorCases; it does not execute a case.
// https://dom.spec.whatwg.org/#interface-characterdata
// https://dom.spec.whatwg.org/#interface-processinginstruction
// https://webidl.spec.whatwg.org/#es-attributes
// https://html.spec.whatwg.org/multipage/parsing.html#serialising-html-fragments
// Complete pseudo-attribute APIs and lone-surrogate data storage are gaps,
// not success cases in this fixture.
var piAccessorCases = (function () {
    function check(value, label) { if (!value) throw new Error(label); }
    function typeError(action) {
        var error;
        try { action(); } catch (caught) { error = caught; }
        check(error instanceof TypeError, 'TypeError');
    }
    function descriptor(owner, key, writable) {
        var d = Object.getOwnPropertyDescriptor(owner, key);
        check(d !== undefined && typeof d.get === 'function', key + ' own getter');
        check(d.value === undefined && d.writable === undefined, key + ' accessor kind');
        check(d.enumerable === true && d.configurable === true, key + ' flags');
        check(d.get.name === 'get ' + key && d.get.length === 0, key + ' getter metadata');
        if (writable) {
            check(typeof d.set === 'function' && d.set.name === 'set ' + key && d.set.length === 1,
                  key + ' setter metadata');
        } else check(d.set === undefined, key + ' no setter');
        return d;
    }
    return {
        descriptors_and_placement: function () {
            var data = descriptor(CharacterData.prototype, 'data', true);
            descriptor(CharacterData.prototype, 'length', false);
            descriptor(ProcessingInstruction.prototype, 'target', false);
            var nodes = [new Text('x'), new Comment('y'), new ProcessingInstruction('probe', 'z')];
            for (var i = 0; i < nodes.length; i++) {
                check(Object.getOwnPropertyDescriptor(nodes[i], 'data') === undefined,
                      'data is inherited, not a copied own property');
                check(Object.getOwnPropertyDescriptor(nodes[i], 'length') === undefined,
                      'length is inherited');
            }
            check(Object.getOwnPropertyDescriptor(Text.prototype, 'data') === undefined &&
                  Object.getOwnPropertyDescriptor(Comment.prototype, 'data') === undefined &&
                  Object.getOwnPropertyDescriptor(ProcessingInstruction.prototype, 'data') === undefined,
                  'shared CharacterData placement');
            check(Object.getOwnPropertyDescriptor(nodes[2], 'target') === undefined,
                  'target is inherited');
            check(Object.getOwnPropertyDescriptor(CharacterData.prototype, 'data').get === data.get,
                  'stable getter identity');
            return true;
        },

        scalar_data_and_setter_defaults: function () {
            var d = descriptor(CharacterData.prototype, 'data', true);
            var nodes = [new Text(''), new Comment(''), new ProcessingInstruction('probe')];
            for (var i = 0; i < nodes.length; i++) {
                var n = nodes[i];
                check(d.set.call(n, 'A\uD834\uDD1E\u0000B') === undefined, 'setter returns undefined');
                check(n.data === 'A\uD834\uDD1E\u0000B' && n.length === 5,
                      'UTF16 units, including a scalar pair and NUL');
                check(n.textContent === n.data, 'native textContent alias');
                n.data = null;
                check(n.data === '' && n.length === 0, 'LegacyNullToEmptyString');
                n.data = undefined;
                check(n.data === 'undefined' && n.length === 9, 'explicit undefined');
                d.set.call(n);
                check(n.data === 'undefined', 'missing setter argument converts undefined');
                typeError(function () { d.set.call(n, Symbol('bad')); });
                check(n.data === 'undefined', 'failed conversion leaves data');
            }
            check(new ProcessingInstruction('probe', undefined).data === '' &&
                  new ProcessingInstruction('probe', null).data === 'null',
                  'constructor optional default differs from attribute setter');
            return true;
        },

        authentic_brands_before_conversion: function () {
            var d = descriptor(CharacterData.prototype, 'data', true);
            var l = descriptor(CharacterData.prototype, 'length', false);
            var t = descriptor(ProcessingInstruction.prototype, 'target', false);
            var calls = 0;
            var value = {toString: function () { calls++; return 'unexpected'; }};
            var wrong = [undefined, null, {}, Object.create(CharacterData.prototype),
                         document, document.createElement('i'), new DocumentFragment()];
            for (var i = 0; i < wrong.length; i++) {
                var receiver = wrong[i];
                typeError(function () { d.get.call(receiver); });
                typeError(function () { l.get.call(receiver); });
                typeError(function () { d.set.call(receiver, value); });
            }
            typeError(function () { t.get.call(new Text('text')); });
            typeError(function () { t.get.call(new Comment('comment')); });
            typeError(function () { t.get.call(Object.create(ProcessingInstruction.prototype)); });
            check(calls === 0, 'invalid receiver rejects before ToString');
            var real = new Text('kept');
            Object.setPrototypeOf(real, null);
            check(d.get.call(real) === 'kept', 'authentic slot survives prototype change');
            d.set.call(real, 'next');
            check(d.get.call(real) === 'next' && l.get.call(real) === 4,
                  'saved accessors accept genuine altered-prototype node');
            return true;
        },

        setter_callback_order_and_abrupt_identity: function () {
            var d = descriptor(CharacterData.prototype, 'data', true);
            var text = new Text('old'), comment = new Comment('untouched');
            var trace = '';
            var value = {};
            value[Symbol.toPrimitive] = function (hint) {
                check(this === value && hint === 'string' && text.data === 'old',
                      'conversion receiver, hint, and pre-write state');
                trace += 'C';
                text.data = 'inner';
                comment.data = 'side';
                return 'outer\uD83D\uDE00';
            };
            d.set.call(text, value);
            check(trace === 'C' && text.data === 'outer\uD83D\uDE00' && text.length === 7 &&
                  comment.data === 'side', 'conversion side effects before final replacement');
            var marker = {}, caught;
            value[Symbol.toPrimitive] = function () {
                text.data = 'prefix';
                throw marker;
            };
            try { d.set.call(text, value); } catch (error) { caught = error; }
            check(caught === marker && text.data === 'prefix',
                  'throw identity and conversion prefix retained without outer write');
            return true;
        },

        own_shadows_and_readonly_assignments: function () {
            var d = descriptor(CharacterData.prototype, 'data', true);
            var l = descriptor(CharacterData.prototype, 'length', false);
            var t = descriptor(ProcessingInstruction.prototype, 'target', false);
            var pi = new ProcessingInstruction('probe', 'kept'), calls = 0;
            var value = {toString: function () { calls++; return 'converted'; }};
            typeError(function () { 'use strict'; pi.length = value; });
            typeError(function () { 'use strict'; pi.target = value; });
            check(calls === 0 && pi.length === 4 && pi.target === 'probe',
                  'readonly strict assignment does not convert or write');
            Object.defineProperty(pi, 'data', {value: 'shadow', writable: true, configurable: true});
            pi.data = value;
            check(pi.data === value && calls === 0 && d.get.call(pi) === 'kept',
                  'ordinary own shadow bypasses native setter');
            Object.defineProperty(pi, 'length', {value: 99, configurable: true});
            Object.defineProperty(pi, 'target', {value: 'shadow-target', configurable: true});
            check(pi.length === 99 && l.get.call(pi) === 4 && pi.target === 'shadow-target' &&
                  t.get.call(pi) === 'probe', 'readonly inherited getters can be shadowed');
            check(delete pi.data && delete pi.length && delete pi.target, 'delete own shadows');
            check(pi.data === 'kept' && pi.length === 4 && pi.target === 'probe',
                  'deletion restores native inherited accessors');
            return true;
        },

        prototype_replacement_deletion_and_saved_getters: function () {
            var pi = new ProcessingInstruction('probe', 'kept');
            var owners = [CharacterData.prototype, CharacterData.prototype, ProcessingInstruction.prototype];
            var keys = ['data', 'length', 'target'], expected = ['kept', 4, 'probe'];
            for (var i = 0; i < keys.length; i++) {
                var owner = owners[i], key = keys[i];
                var original = Object.getOwnPropertyDescriptor(owner, key), token = {};
                try {
                    Object.defineProperty(owner, key, {
                        get: function () { check(this === pi, 'original getter receiver'); return token; },
                        configurable: true
                    });
                    check(pi[key] === token && original.get.call(pi) === expected[i],
                          'prototype replacement and saved native getter');
                    check(delete owner[key], 'delete prototype property');
                    check(pi[key] === undefined && original.get.call(pi) === expected[i],
                          'deleted accessors do not resurrect virtual getters');
                } finally { Object.defineProperty(owner, key, original); }
                check(pi[key] === expected[i], 'explicit descriptor restoration');
            }
            return true;
        },

        pi_setter_clone_and_html_serialization: function () {
            var pi = new ProcessingInstruction('probe', 'first');
            pi.data = '?>';
            check(pi.data === '?>' && pi.length === 2,
                  'CharacterData mutation does not rerun creation validation');
            var clone = pi.cloneNode(false), holder = document.createElement('section');
            check(clone !== pi && clone.data === '?>' && clone.target === 'probe' && clone.nodeType === 7,
                  'clone copies PI slots without constructor validation');
            document.append(clone);
            check(clone.parentNode === document, 'PI is a permitted Document child');
            holder.appendChild(clone);
            check(holder.innerHTML === '<?probe ?>?>', 'literal HTML PI serialization');
            clone.data = 'end?';
            check(holder.innerHTML === '<?probe end??>' && pi.data === '?>',
                  'trailing question mark and clone isolation');
            return true;
        }
    };
})();
