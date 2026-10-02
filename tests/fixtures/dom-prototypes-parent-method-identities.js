// Independent small case; all nine defining-interface identities, without mutation.
(function () {
    function check(value, message) { if (!value) throw new Error(message); }
    var owners = [Document.prototype, Element.prototype, DocumentFragment.prototype];
    var element = document.createElement('section'), fragment = document.createDocumentFragment();
    var peers = [document, document.createElement('aside'), document.createDocumentFragment()];
    var targets = [document, element, fragment], names = ['querySelector', 'querySelectorAll', 'append'];
    var saved = [];
    for (var i = 0; i < owners.length; i++) {
        for (var j = 0; j < names.length; j++) {
            var key = names[j], method = owners[i][key];
            check(typeof method === 'function', 'own interface operation');
            check(targets[i][key] === method && peers[i][key] === method,
                  'one function shared within defining interface');
            check(Object.getOwnPropertyDescriptor(targets[i], key) === undefined &&
                  Object.getOwnPropertyDescriptor(Node.prototype, key) === undefined,
                  'mixin operation has correct placement');
            for (var k = 0; k < saved.length; k++) check(saved[k] !== method, 'nine distinct identities');
            saved.push(method);
        }
    }
    check(saved.length === 9, 'complete identity inventory');
    return true;
})();
