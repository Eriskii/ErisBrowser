// Independent bounded category: authentic constructor override.
// Execute this complete IIFE in a fresh Runtime/Document, in each source mode.
// The original piAccessorCases.authentic_brands_before_conversion remains
// byte-exact and ordinarily expected to succeed; host Object.setPrototypeOf
// there is a separately retained unsupported prerequisite.
// https://webidl.spec.whatwg.org/#internally-create-a-new-object-implementing-the-interface
// https://webidl.spec.whatwg.org/#es-attributes
(function () {
    function check(value, label) { if (!value) throw new Error(label); }
    function typeError(action) {
        var error;
        try { action(); } catch (caught) { error = caught; }
        check(error instanceof TypeError, 'TypeError');
    }
    var d = Object.getOwnPropertyDescriptor(CharacterData.prototype, 'data');
    var l = Object.getOwnPropertyDescriptor(CharacterData.prototype, 'length');
    var t = Object.getOwnPropertyDescriptor(ProcessingInstruction.prototype, 'target');
    check(typeof d.get === 'function' && typeof d.set === 'function' &&
          typeof l.get === 'function' && typeof t.get === 'function', 'saved native accessors');
    var pi = new ProcessingInstruction('probe', 'data');
    check(t.get.call(pi) === 'probe' && d.get.call(pi) === 'data', 'successful PI accessors');
    var calls = 0, poison = {toString: function () { calls++; return 'bad'; }};
    var forged = Object.create(Text.prototype);
    check(forged instanceof Text, 'prototype membership alone is insufficient');
    typeError(function () { d.get.call(forged); });
    typeError(function () { l.get.call(forged); });
    typeError(function () { d.set.call(forged, poison); });
    typeError(function () { d.set.call(document, poison); });
    typeError(function () { t.get.call(Object.create(ProcessingInstruction.prototype)); });
    check(calls === 0, 'brand failure precedes setter conversion');

    var trace = '', custom = Object.create(null);
    var NewTarget = (function () {}).bind(null);
    Object.defineProperty(NewTarget, 'prototype', {
        get: function () { trace += 'P'; return custom; }, configurable: true
    });
    var real = Reflect.construct(Text, [{toString: function () { trace += 'C'; return 'kept'; }}], NewTarget);
    check(trace === 'CP' && Object.getPrototypeOf(real) === custom,
          'conversion then explicit constructor prototype selection');
    check(!(real instanceof Text) && d.get.call(real) === 'kept' && l.get.call(real) === 4,
          'genuine slots survive absent Text prototype membership');
    trace = '';
    d.set.call(real, {toString: function () { trace += 'S'; return '\uD83D\uDE00'; }});
    check(trace === 'S' && d.get.call(real) === '\uD83D\uDE00' && l.get.call(real) === 2,
          'saved native setter accepts genuine alternate-prototype node');
    typeError(function () { t.get.call(real); });
    return true;
})()
