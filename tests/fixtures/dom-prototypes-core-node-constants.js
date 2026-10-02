// Independent small case; constants retain the literal values from core-fixture.js.
(function () {
    function check(value, message) { if (!value) throw new Error(message); }
    var rows = [
        ['ELEMENT_NODE', 1], ['ATTRIBUTE_NODE', 2], ['TEXT_NODE', 3], ['CDATA_SECTION_NODE', 4],
        ['ENTITY_REFERENCE_NODE', 5], ['ENTITY_NODE', 6], ['PROCESSING_INSTRUCTION_NODE', 7],
        ['COMMENT_NODE', 8], ['DOCUMENT_NODE', 9], ['DOCUMENT_TYPE_NODE', 10],
        ['DOCUMENT_FRAGMENT_NODE', 11], ['NOTATION_NODE', 12],
        ['DOCUMENT_POSITION_DISCONNECTED', 1], ['DOCUMENT_POSITION_PRECEDING', 2],
        ['DOCUMENT_POSITION_FOLLOWING', 4], ['DOCUMENT_POSITION_CONTAINS', 8],
        ['DOCUMENT_POSITION_CONTAINED_BY', 16], ['DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC', 32]
    ];
    var owners = [Node, Node.prototype];
    for (var i = 0; i < rows.length; i++) {
        for (var j = 0; j < owners.length; j++) {
            var d = Object.getOwnPropertyDescriptor(owners[j], rows[i][0]);
            check(d !== undefined && d.value === rows[i][1], 'constant literal value');
            check(d.writable === false && d.enumerable === true && d.configurable === false &&
                  d.get === undefined && d.set === undefined, 'constant descriptor');
        }
    }
    return true;
})();
