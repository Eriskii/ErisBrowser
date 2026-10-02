// Independent next-step oracle; not part of the current prototype feature.
// Execute ONE named piFollowupCases case per fresh Runtime and Document.
// Loading this file defines cases; it does not run them. Each returns true.
// https://dom.spec.whatwg.org/#interface-processinginstruction
// https://dom.spec.whatwg.org/#dom-document-createprocessinginstruction
// https://www.w3.org/TR/xml/#NT-Name
// https://webidl.spec.whatwg.org/#es-interface-call
// https://webidl.spec.whatwg.org/#es-operations
// Excluded success claims: complete pseudo-attribute methods, lone-surrogate
// data storage, independent Document construction, and cross-realm NewTarget.
var piFollowupCases = (function () {
    function check(condition, label) {
        if (!condition) throw new Error(label);
    }
    function typeError(action, label) {
        var caught = false;
        try { action(); } catch (error) {
            caught = true;
            check(error instanceof TypeError, label + ': TypeError identity');
        }
        check(caught, label + ': did not throw');
    }
    function invalidCharacter(action, label) {
        var caught = false;
        try { action(); } catch (error) {
            caught = true;
            check(error instanceof DOMException, label + ': DOMException identity');
            check(error.name === 'InvalidCharacterError', label + ': exception name');
            check(error.code === 5, label + ': legacy exception code');
        }
        check(caught, label + ': did not throw');
    }
    function throwsSame(marker, action, label) {
        var caught = false;
        try { action(); } catch (error) {
            caught = true;
            check(error === marker, label + ': abrupt identity');
        }
        check(caught, label + ': did not throw');
    }
    function alternateTarget(getPrototype) {
        // A bound ordinary constructor has no preexisting own prototype, so
        // its observable accessor can be installed without redefining a
        // nonconfigurable ordinary function prototype property.
        var target = (function () {}).bind(null);
        Object.defineProperty(target, 'prototype', {
            get: getPrototype,
            configurable: true
        });
        return target;
    }
    return {
        constructor_arity_defaults_and_call: function () {
            check(typeof ProcessingInstruction === 'function', 'PI constructor available');
            check(ProcessingInstruction.length === 1, 'PI constructor arity');
            typeError(function () { new ProcessingInstruction(); }, 'missing target');
            var trace = '';
            var value = {toString: function () { trace += 'C'; return 'probe'; }};
            typeError(function () { ProcessingInstruction(value, value); }, 'plain call');
            check(trace === '', 'plain call must not coerce arguments');
            var empty = new ProcessingInstruction('probe');
            var undef = new ProcessingInstruction('probe', undefined);
            var explicitTarget = new ProcessingInstruction(undefined, null);
            check(empty.data === '' && undef.data === '', 'constructor data default');
            check(explicitTarget.target === 'undefined' && explicitTarget.data === 'null',
                  'required undefined target and explicit null data');
            var unused = {toString: function () { throw new Error('extra argument coerced'); }};
            check(new ProcessingInstruction('probe', 'kept', unused).data === 'kept',
                  'extra argument ignored');
            return true;
        },

        factory_brand_arity_and_required_data: function () {
            var factory = document.createProcessingInstruction;
            check(typeof factory === 'function' && factory.length === 2, 'factory arity');
            var trace = '';
            var value = {toString: function () { trace += 'C'; return 'probe'; }};
            typeError(function () { factory.call(document); }, 'factory missing arguments');
            typeError(function () { factory.call(document, value); }, 'factory missing data');
            check(trace === '', 'arity failure before conversion');
            typeError(function () { factory.call({}, value, value); }, 'factory forged receiver');
            typeError(function () {
                factory.call(document.createDocumentFragment(), value, value);
            }, 'factory wrong node interface');
            check(trace === '', 'factory brand before conversion');
            typeError(function () { new factory(value, value); }, 'factory is not constructor');
            check(trace === '', 'factory construction failure before conversion');
            var node = factory.call(document, 'probe', undefined);
            check(node.target === 'probe' && node.data === 'undefined', 'required data conversion');
            return true;
        },

        xml_name_valid_targets: function () {
            // DOM initialization uses XML Name, not XML PITarget and not the
            // HTML tokenizer's narrower/reserved-name rules.
            var names = [':', 'a:b:c', '_name', 'xml', 'XML', 'xml-stylesheet',
                         '\u00e9', '\u03a9', 'a\u0300'];
            for (var i = 0; i < names.length; i++) {
                var a = new ProcessingInstruction(names[i], 'x');
                var b = document.createProcessingInstruction(names[i], 'y');
                check(a.target === names[i] && b.target === names[i], 'literal XML Name');
            }
            return true;
        },

        xml_name_invalid_targets: function () {
            var names = ['', '0name', '-name', '.name', 'a b', 'a/b', '\u0300a', '\u0000'];
            for (var i = 0; i < names.length; i++) {
                var name = names[i];
                invalidCharacter(function () { new ProcessingInstruction(name, ''); },
                                 'constructor invalid Name');
                invalidCharacter(function () { document.createProcessingInstruction(name, ''); },
                                 'factory invalid Name');
            }
            return true;
        },

        xml_name_unicode_boundaries: function () {
            var valid = ['\u037d', '\u037f', '\uD800\uDC00', '\uDB7F\uDFFF'];
            for (var i = 0; i < valid.length; i++) {
                check(new ProcessingInstruction(valid[i], '').target === valid[i],
                      'XML Name Unicode boundary');
            }
            // U+037E is a range gap; U+F0000 exceeds Name's astral ceiling.
            // A lone surrogate is an invalid target, not a successful-storage case.
            var invalid = ['\u037e', '\uDB80\uDC00', '\uD800'];
            for (var j = 0; j < invalid.length; j++) {
                var name = invalid[j];
                invalidCharacter(function () { new ProcessingInstruction(name, ''); },
                                 'invalid Unicode target');
            }
            return true;
        },

        data_terminator_and_literal_storage: function () {
            var data = ['', '?', 'a>b', 'a?b', '\u0000', '\u00e9\uD834\uDD1E'];
            for (var i = 0; i < data.length; i++) {
                var a = new ProcessingInstruction('probe', data[i]);
                var b = document.createProcessingInstruction('probe', data[i]);
                check(a.data === data[i] && b.data === data[i], 'literal data preserved');
            }
            var forbidden = ['?>', 'before?>after'];
            for (var j = 0; j < forbidden.length; j++) {
                var text = forbidden[j];
                invalidCharacter(function () { new ProcessingInstruction('probe', text); },
                                 'constructor terminator');
                invalidCharacter(function () { document.createProcessingInstruction('probe', text); },
                                 'factory terminator');
            }
            return true;
        },

        genuine_node_identity_and_data: function () {
            var a = new ProcessingInstruction('probe', 'first');
            var b = document.createProcessingInstruction('probe', 'second');
            check(a !== b, 'fresh node identities');
            var nodes = [a, b];
            for (var i = 0; i < nodes.length; i++) {
                var node = nodes[i];
                check(Object.getPrototypeOf(node) === ProcessingInstruction.prototype,
                      'initial PI prototype');
                check(node instanceof ProcessingInstruction && node instanceof CharacterData &&
                      node instanceof Node, 'PI inheritance');
                check(node.nodeType === 7 && node.nodeName === 'probe' && node.target === 'probe',
                      'real PI node identity');
                check(node.parentNode === null, 'new PI is detached');
                check(node.textContent === node.data, 'native data visible as textContent');
            }
            check(a.data === 'first' && b.data === 'second', 'distinct retained data');
            return true;
        },

        conversions_then_alternate_prototype: function () {
            var trace = '';
            var custom = Object.create(ProcessingInstruction.prototype);
            var target = alternateTarget(function () { trace += 'P'; return custom; });
            var name = {toString: function () { trace += 'T'; return 'probe'; }};
            var data = {toString: function () { trace += 'D'; return 'kept'; }};
            var node = Reflect.construct(ProcessingInstruction, [name, data], target);
            check(trace === 'TDP', 'target then data then prototype');
            check(Object.getPrototypeOf(node) === custom, 'alternate prototype selected');
            check(node.nodeType === 7 && node.target === 'probe' && node.data === 'kept',
                  'alternate prototype retains authentic PI state');
            trace = '';
            var fallback = alternateTarget(function () { trace += 'P'; return 17; });
            var ordinary = Reflect.construct(ProcessingInstruction, ['probe'], fallback);
            check(trace === 'P' && Object.getPrototypeOf(ordinary) === ProcessingInstruction.prototype,
                  'primitive prototype selects intrinsic PI prototype');
            return true;
        },

        validation_follows_conversions_and_prototype: function () {
            var trace = '';
            var target = alternateTarget(function () { trace += 'P'; return ProcessingInstruction.prototype; });
            var name = {toString: function () { trace += 'T'; return 'bad name'; }};
            var data = {toString: function () { trace += 'D'; return 'kept'; }};
            invalidCharacter(function () {
                Reflect.construct(ProcessingInstruction, [name, data], target);
            }, 'invalid target validation');
            check(trace === 'TDP', 'invalid target does not skip conversions/prototype');
            trace = '';
            name = {toString: function () { trace += 'T'; return 'probe'; }};
            data = {toString: function () { trace += 'D'; return '?>'; }};
            invalidCharacter(function () {
                Reflect.construct(ProcessingInstruction, [name, data], target);
            }, 'invalid data validation');
            check(trace === 'TDP', 'invalid data does not skip prototype');
            return true;
        },

        conversion_abrupt_identity: function () {
            var marker = {}, trace = '';
            var target = alternateTarget(function () { trace += 'P'; return ProcessingInstruction.prototype; });
            var name = {toString: function () { trace += 'T'; throw marker; }};
            var data = {toString: function () { trace += 'D'; return 'kept'; }};
            throwsSame(marker, function () {
                Reflect.construct(ProcessingInstruction, [name, data], target);
            }, 'target conversion throw');
            check(trace === 'T', 'target throw stops data/prototype');
            trace = '';
            name = {toString: function () { trace += 'T'; return 'bad name'; }};
            data = {toString: function () { trace += 'D'; throw marker; }};
            throwsSame(marker, function () {
                Reflect.construct(ProcessingInstruction, [name, data], target);
            }, 'data conversion beats invalid target');
            check(trace === 'TD', 'data throw stops prototype/validation');
            return true;
        },

        prototype_abrupt_precedes_validation: function () {
            var marker = {}, trace = '';
            var target = alternateTarget(function () { trace += 'P'; throw marker; });
            var name = {toString: function () { trace += 'T'; return 'bad name'; }};
            var data = {toString: function () { trace += 'D'; return '?>'; }};
            throwsSame(marker, function () {
                Reflect.construct(ProcessingInstruction, [name, data], target);
            }, 'prototype throw beats content validation');
            check(trace === 'TDP', 'prototype abrupt order');
            return true;
        },

        factory_conversion_order_and_abrupt_identity: function () {
            var marker = {}, trace = '';
            var name = {toString: function () { trace += 'T'; return 'probe'; }};
            var data = {toString: function () { trace += 'D'; return 'kept'; }};
            var node = document.createProcessingInstruction(name, data);
            check(trace === 'TD' && node.target === 'probe' && node.data === 'kept',
                  'factory conversion order');
            trace = '';
            name = {toString: function () { trace += 'T'; return 'bad name'; }};
            data = {toString: function () { trace += 'D'; throw marker; }};
            throwsSame(marker, function () {
                document.createProcessingInstruction(name, data);
            }, 'factory data conversion beats target validation');
            check(trace === 'TD', 'factory abrupt conversion order');
            return true;
        }
    };
})();
