(function () {
    if (typeof String !== 'function' || typeof Number !== 'function' || String(1.25) !== '1.25') {
        throw new Error('formatter witness prerequisite');
    }
    var n = 1000000000000000.25;
    if (n - 1000000000000000 !== 0.25 || (n + 0.125) - n !== 0.125 ||
        Number('1000000000000000.2') !== n || Number('1000000000000000.3') !== n) {
        throw new Error('exact midpoint prerequisite');
    }
    if (String(n) !== '1000000000000000.3') throw new Error('wrong shortest tie choice');
    return true;
})()
