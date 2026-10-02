// Independent ordinary-success regression draft. Run unchanged in sloppy and strict modes.
// Requires the supported Document/Element/DocumentFragment querySelector methods;
// does not require new Document(), interface globals or mutable interface prototypes.
(function () {
  function check(value, message) { if (!value) throw new Error(message); }
  function dataDescriptor(object, key, value, writable, enumerable, configurable) {
    var d = Object.getOwnPropertyDescriptor(object, key);
    check(d !== undefined && d.value === value, 'own descriptor value');
    check(d.writable === writable && d.enumerable === enumerable && d.configurable === configurable, 'own descriptor flags');
    check(d.get === undefined && d.set === undefined, 'data descriptor kind');
  }
  var element = document.createElement('section'), sibling = document.createElement('section');
  var fragment = document.createDocumentFragment(), otherFragment = document.createDocumentFragment();
  var docProbe = document.createElement('i'), elementProbe = document.createElement('i'), fragmentProbe = document.createElement('i');
  docProbe.id = 'erisOwnPropertyProbe'; document.body.appendChild(docProbe);
  element.appendChild(elementProbe); fragment.appendChild(fragmentProbe);
  var targets = [document, element, fragment];
  var saved = [document.querySelector, element.querySelector, fragment.querySelector];
  var selectors = ['#erisOwnPropertyProbe', 'i', 'i'];
  var expected = [docProbe, elementProbe, fragmentProbe];
  check(typeof Object.getOwnPropertyDescriptor === 'function' && typeof Object.defineProperty === 'function', 'descriptor prerequisites');
  for (var i = 0; i < 3; i++) {
    var target = targets[i], nativeMethod = saved[i];
    check(typeof nativeMethod === 'function' && nativeMethod.call(target, selectors[i]) === expected[i], 'native operation prerequisite');
    check(Object.getOwnPropertyDescriptor(target, 'querySelector') === undefined, 'method initially inherited');
    var replacement = function () { check(this === target, 'replacement receiver'); return 73; };
    target.querySelector = replacement;
    check(target.querySelector === replacement && target.querySelector() === 73, 'assignment shadows method');
    dataDescriptor(target, 'querySelector', replacement, true, true, true);
    check(Object.prototype.hasOwnProperty.call(target, 'querySelector'), 'hasOwnProperty sees shadow');
    check(Object.prototype.propertyIsEnumerable.call(target, 'querySelector'), 'shadow enumerable');
    for (var j = 0; j < 3; j++) if (j !== i) check(targets[j].querySelector === saved[j], 'cross-interface isolation');
    check(sibling.querySelector === saved[1] && otherFragment.querySelector === saved[2], 'same-interface sibling isolation');
    check(delete target.querySelector, 'delete own shadow');
    check(Object.getOwnPropertyDescriptor(target, 'querySelector') === undefined && target.querySelector === nativeMethod, 'delete reveals native identity');

    Object.defineProperty(target, 'querySelector', {value: replacement, writable: true, enumerable: false, configurable: true});
    dataDescriptor(target, 'querySelector', replacement, true, false, true);
    check(target.querySelector() === 73 && delete target.querySelector, 'defineProperty replacement');
    check(target.querySelector === nativeMethod, 'native identity restored twice');

    var seenGet, seenSet, stored, reads = 0;
    var getter = function () { seenGet = this; reads++; return replacement; };
    var setter = function (value) { seenSet = this; stored = value; };
    Object.defineProperty(target, 'querySelector', {get: getter, set: setter, enumerable: true, configurable: true});
    var descriptor = Object.getOwnPropertyDescriptor(target, 'querySelector');
    check(reads === 0 && descriptor.get === getter && descriptor.set === setter, 'reflection does not invoke accessor');
    check(descriptor.enumerable && descriptor.configurable && descriptor.value === undefined && descriptor.writable === undefined, 'accessor descriptor flags');
    check(target.querySelector === replacement && seenGet === target && reads === 1, 'getter receives public object');
    target.querySelector = 81;
    check(seenSet === target && stored === 81 && reads === 1, 'setter receives public object without getter');
    check(delete target.querySelector && target.querySelector === nativeMethod, 'accessor deletion reveals native');

    var converted = 0, rejected = false;
    try { nativeMethod.call(targets[(i + 1) % 3], {toString: function () { converted++; return 'i'; }}); }
    catch (error) { rejected = error instanceof TypeError; }
    check(rejected && converted === 0, 'saved native keeps defining-interface brand before conversion');
    check(nativeMethod.call(target, selectors[i]) === expected[i], 'saved native still works after shadows');
  }

  // Other key types prevent a method-name-only implementation from passing.
  // Use an unattached fragment to avoid HTML Document named-property behavior.
  var keysTarget = document.createDocumentFragment(), symbol = Symbol('own-property');
  keysTarget[symbol] = 9; // Existing symbol storage must be reused by later string writes.
  keysTarget['9'] = 90; keysTarget['2'] = 20; keysTarget.extra = 1; keysTarget['\uD800'] = 2;
  check(keysTarget[symbol] === 9 && keysTarget['\uD800'] === 2, 'symbol and raw UTF-16 key identity');
  var names = Object.getOwnPropertyNames(keysTarget), keys = Reflect.ownKeys(keysTarget);
  check(names.length === 4 && names[0] === '2' && names[1] === '9' && names[2] === 'extra' && names[3] === '\uD800', 'ordinary string key order');
  check(keys.length === 5 && keys[4] === symbol, 'symbol follows string keys');
  check(delete keysTarget.extra, 'delete arbitrary key'); keysTarget.extra = 3;
  names = Object.getOwnPropertyNames(keysTarget);
  check(names[2] === '\uD800' && names[3] === 'extra', 'reinsertion appends string order');
  check(keysTarget.append === fragment.append && typeof keysTarget.querySelector === 'function', 'own storage preserves native DOM access');
  docProbe.remove();
})();
