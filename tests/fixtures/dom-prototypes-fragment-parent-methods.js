// Independent small case: DocumentFragment's three ParentNode operations.
// Fresh Runtime/Document per execution; no complete Document host enumeration.
(function () {
    function check(value, message) { if (!value) throw new Error(message); }
    function data(object, key, value, writable, enumerable, configurable) {
        var d = Object.getOwnPropertyDescriptor(object, key);
        check(d !== undefined && d.value === value && d.writable === writable &&
              d.enumerable === enumerable && d.configurable === configurable &&
              d.get === undefined && d.set === undefined, 'operation descriptor');
        return d;
    }
    function countKey(object, wanted) {
        var count = 0;
        for (var key in object) if (key === wanted) count++;
        return count;
    }
    var owner = DocumentFragment.prototype, target = document.createDocumentFragment();
    var answer = document.createElement('i'), selector = 'i';
    target.appendChild(answer);
    var enumerableReceiver = target;
    var names = ['querySelector', 'querySelectorAll', 'append'];
    function invoke(method, key) {
        if (key === 'append') check(method.call(target) === undefined, 'saved append');
        else if (key === 'querySelector') check(method.call(target, selector) === answer, 'saved query');
        else {
            var list = method.call(target, selector);
            check(list.length === 1 && list[0] === answer, 'saved query all');
        }
    }
    var bag = owner[Symbol.unscopables];
    data(owner, Symbol.unscopables, bag, false, false, true);
    check(Object.getPrototypeOf(bag) === null, 'null-prototype unscopables');
    data(bag, 'append', true, true, true, true);
    for (var i = 0; i < names.length; i++) {
        var key = names[i], method = target[key];
        check(typeof method === 'function', 'native operation callable');
        var original = data(owner, key, method, true, true, true);
        data(method, 'name', key, false, false, true);
        data(method, 'length', key === 'append' ? 0 : 1, false, false, true);
        check(Object.getOwnPropertyDescriptor(target, key) === undefined, 'no instance copy');
        invoke(method, key);
        var forged = Object.create(owner), conversions = 0, error = undefined;
        check(owner.isPrototypeOf(forged) && forged instanceof owner.constructor,
              'forged prototype membership');
        try {
            method.call(forged, {toString: function () { conversions++; return 'i'; }});
        } catch (caught) { error = caught; }
        check(error instanceof TypeError && conversions === 0, 'brand before coercion');
        var marker = {}, replacement = function () {
            check(this === target, 'replacement receiver');
            return marker;
        };
        try {
            owner[key] = replacement;
            check(target[key] === replacement && target[key]() === marker, 'replacement visible');
            check(Object.getOwnPropertyDescriptor(target, key) === undefined, 'replacement inherited');
            check(delete owner[key], 'prototype operation configurable');
            check(target[key] === undefined && !(key in target) &&
                  Object.getOwnPropertyDescriptor(owner, key) === undefined, 'no fallback resurrection');
            if (i === 0) check(countKey(enumerableReceiver, key) === 0, 'deleted operation not enumerated');
            invoke(method, key);
        } finally {
            Object.defineProperty(owner, key, original);
        }
        check(target[key] === method, 'explicit restoration');
        if (i === 0) check(countKey(enumerableReceiver, key) === 1, 'restored operation enumerated');
    }
    return true;
})();
