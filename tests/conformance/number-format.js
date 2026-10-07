// Independent raw-bit/literal expectations from the held 74-row algebra fixture.
var numberFormatRows = [
  [
    "positive_zero",
    0,
    0,
    "0"
  ],
  [
    "negative_zero",
    2147483648,
    0,
    "0"
  ],
  [
    "positive_infinity",
    2146435072,
    0,
    "Infinity"
  ],
  [
    "negative_infinity",
    4293918720,
    0,
    "-Infinity"
  ],
  [
    "quiet_nan",
    2146959360,
    0,
    "NaN"
  ],
  [
    "negative_quiet_nan",
    4294443008,
    1,
    "NaN"
  ],
  [
    "one",
    1072693248,
    0,
    "1"
  ],
  [
    "negative_one",
    3220176896,
    0,
    "-1"
  ],
  [
    "quarter",
    1070596096,
    0,
    "0.25"
  ],
  [
    "negative_quarter",
    3218079744,
    0,
    "-0.25"
  ],
  [
    "one_quarter",
    1072955392,
    0,
    "1.25"
  ],
  [
    "negative_one_quarter",
    3220439040,
    0,
    "-1.25"
  ],
  [
    "one_half",
    1073217536,
    0,
    "1.5"
  ],
  [
    "negative_one_half",
    3220701184,
    0,
    "-1.5"
  ],
  [
    "tenth",
    1069128089,
    2576980378,
    "0.1"
  ],
  [
    "negative_tenth",
    3216611737,
    2576980378,
    "-0.1"
  ],
  [
    "e15_integer",
    1124887541,
    640942080,
    "1000000000000000"
  ],
  [
    "negative_e15_integer",
    3272371189,
    640942080,
    "-1000000000000000"
  ],
  [
    "e15_lower_neighbor",
    1124887541,
    640942081,
    "1000000000000000.1"
  ],
  [
    "negative_e15_lower_neighbor",
    3272371189,
    640942081,
    "-1000000000000000.1"
  ],
  [
    "e15_odd_upward_tie",
    1124887541,
    640942082,
    "1000000000000000.2"
  ],
  [
    "negative_e15_odd_upward_tie",
    3272371189,
    640942082,
    "-1000000000000000.2"
  ],
  [
    "e15_upper_neighbor",
    1124887541,
    640942083,
    "1000000000000000.4"
  ],
  [
    "negative_e15_upper_neighbor",
    3272371189,
    640942083,
    "-1000000000000000.4"
  ],
  [
    "e15_exact_half",
    1124887541,
    640942084,
    "1000000000000000.5"
  ],
  [
    "negative_e15_exact_half",
    3272371189,
    640942084,
    "-1000000000000000.5"
  ],
  [
    "e15_second_lower_neighbor",
    1124887541,
    640942085,
    "1000000000000000.6"
  ],
  [
    "negative_e15_second_lower_neighbor",
    3272371189,
    640942085,
    "-1000000000000000.6"
  ],
  [
    "e15_already_even_upward_tie",
    1124887541,
    640942086,
    "1000000000000000.8"
  ],
  [
    "negative_e15_already_even_upward_tie",
    3272371189,
    640942086,
    "-1000000000000000.8"
  ],
  [
    "e15_second_upper_neighbor",
    1124887541,
    640942087,
    "1000000000000000.9"
  ],
  [
    "negative_e15_second_upper_neighbor",
    3272371189,
    640942087,
    "-1000000000000000.9"
  ],
  [
    "e15_next_integer",
    1124887541,
    640942088,
    "1000000000000001"
  ],
  [
    "negative_e15_next_integer",
    3272371189,
    640942088,
    "-1000000000000001"
  ],
  [
    "below_lower_notation_threshold",
    1051772663,
    2696277388,
    "9.999999999999997e-7"
  ],
  [
    "negative_below_lower_notation_threshold",
    3199256311,
    2696277388,
    "-9.999999999999997e-7"
  ],
  [
    "lower_notation_threshold",
    1051772663,
    2696277389,
    "0.000001"
  ],
  [
    "negative_lower_notation_threshold",
    3199256311,
    2696277389,
    "-0.000001"
  ],
  [
    "above_lower_notation_threshold",
    1051772663,
    2696277390,
    "0.0000010000000000000002"
  ],
  [
    "negative_above_lower_notation_threshold",
    3199256311,
    2696277390,
    "-0.0000010000000000000002"
  ],
  [
    "one_e_minus_seven",
    1048238066,
    2596056904,
    "1e-7"
  ],
  [
    "negative_one_e_minus_seven",
    3195721714,
    2596056904,
    "-1e-7"
  ],
  [
    "one_e_twenty",
    1142271773,
    2025163840,
    "100000000000000000000"
  ],
  [
    "negative_one_e_twenty",
    3289755421,
    2025163840,
    "-100000000000000000000"
  ],
  [
    "below_upper_notation_threshold",
    1145772772,
    3605196623,
    "999999999999999900000"
  ],
  [
    "negative_below_upper_notation_threshold",
    3293256420,
    3605196623,
    "-999999999999999900000"
  ],
  [
    "upper_notation_threshold",
    1145772772,
    3605196624,
    "1e+21"
  ],
  [
    "negative_upper_notation_threshold",
    3293256420,
    3605196624,
    "-1e+21"
  ],
  [
    "above_upper_notation_threshold",
    1145772772,
    3605196625,
    "1.0000000000000001e+21"
  ],
  [
    "negative_above_upper_notation_threshold",
    3293256420,
    3605196625,
    "-1.0000000000000001e+21"
  ],
  [
    "minimum_subnormal",
    0,
    1,
    "5e-324"
  ],
  [
    "negative_minimum_subnormal",
    2147483648,
    1,
    "-5e-324"
  ],
  [
    "second_subnormal",
    0,
    2,
    "1e-323"
  ],
  [
    "negative_second_subnormal",
    2147483648,
    2,
    "-1e-323"
  ],
  [
    "largest_subnormal",
    1048575,
    4294967295,
    "2.225073858507201e-308"
  ],
  [
    "negative_largest_subnormal",
    2148532223,
    4294967295,
    "-2.225073858507201e-308"
  ],
  [
    "minimum_normal",
    1048576,
    0,
    "2.2250738585072014e-308"
  ],
  [
    "negative_minimum_normal",
    2148532224,
    0,
    "-2.2250738585072014e-308"
  ],
  [
    "maximum_finite_predecessor",
    2146435071,
    4294967294,
    "1.7976931348623155e+308"
  ],
  [
    "negative_maximum_finite_predecessor",
    4293918719,
    4294967294,
    "-1.7976931348623155e+308"
  ],
  [
    "maximum_finite",
    2146435071,
    4294967295,
    "1.7976931348623157e+308"
  ],
  [
    "negative_maximum_finite",
    4293918719,
    4294967295,
    "-1.7976931348623157e+308"
  ],
  [
    "u32_max",
    1106247679,
    4292870144,
    "4294967295"
  ],
  [
    "negative_u32_max",
    3253731327,
    4292870144,
    "-4294967295"
  ],
  [
    "u32_next",
    1106247680,
    0,
    "4294967296"
  ],
  [
    "negative_u32_next",
    3253731328,
    0,
    "-4294967296"
  ],
  [
    "safe_integer_max",
    1128267775,
    4294967295,
    "9007199254740991"
  ],
  [
    "negative_safe_integer_max",
    3275751423,
    4294967295,
    "-9007199254740991"
  ],
  [
    "two_pow_53",
    1128267776,
    0,
    "9007199254740992"
  ],
  [
    "negative_two_pow_53",
    3275751424,
    0,
    "-9007199254740992"
  ],
  [
    "two_pow_53_successor",
    1128267776,
    1,
    "9007199254740994"
  ],
  [
    "negative_two_pow_53_successor",
    3275751424,
    1,
    "-9007199254740994"
  ],
  [
    "existing_e18_plus_128",
    1135329645,
    1733216257,
    "1000000000000000100"
  ],
  [
    "negative_existing_e18_plus_128",
    3282813293,
    1733216257,
    "-1000000000000000100"
  ]
];

var numberFormatCases = {
  public_row: function (index) {
    if (typeof DataView !== 'function' || typeof ArrayBuffer !== 'function' ||
        typeof JSON.stringify !== 'function' || String(1.25) !== '1.25' ||
        Number.prototype.toString.call(1.25,10) !== '1.25' ||
        JSON.stringify({x:1},['x']) !== '{"x":1}') throw new Error('route prerequisites');
    var view = new DataView(new ArrayBuffer(8));
    view.setUint32(0,0x3ff40000,false); view.setUint32(4,0,false);
    if (view.getFloat64(0,false) !== 1.25) throw new Error('raw bits prerequisite');
    var row = numberFormatRows[index];
    view.setUint32(0,row[1],false); view.setUint32(4,row[2],false);
    var n = view.getFloat64(0,false), expected = row[3];
    if (String(n) !== expected || Number.prototype.toString.call(n,10) !== expected ||
        ''+n !== expected || `${n}` !== expected) throw new Error('public number spelling');
    var computed = {[n]:7};
    if (Object.keys(computed).length !== 1 || Object.keys(computed)[0] !== expected ||
        computed[expected] !== 7) throw new Error('computed property spelling');
    var assigned = {}; assigned[n] = 'v';
    if (Object.keys(assigned)[0] !== expected) throw new Error('assigned property spelling');
    var json = expected === 'NaN' || expected === 'Infinity' || expected === '-Infinity' ? 'null' : expected;
    if (JSON.stringify(n) !== json || JSON.stringify([n]) !== '['+json+']')
      throw new Error('JSON value spelling');
    var selected = {}; selected[expected] = 'v';
    if (JSON.stringify(selected,[n]) !== '{"'+expected+'":"v"}')
      throw new Error('JSON replacer key spelling');
    return true;
  },
  literal_keys: function () {
    if (Object.keys({1.25:7})[0] !== '1.25' || Function('return 7')() !== 7)
      throw new Error('literal compiler prerequisites');
    var a = {1000000000000000.25:7, 0.000001:8, 1e21:9, 1000000000000000128:10};
    if (Object.keys(a).join('|') !== '1000000000000000.2|0.000001|1e+21|1000000000000000100' ||
        a['1000000000000000.2'] !== 7 || a['1000000000000000.3'] !== undefined)
      throw new Error('compiled literal property keys');
    var b = Function('return {1000000000000000.25:7}')();
    if (Object.keys(b)[0] !== '1000000000000000.2' || b['1000000000000000.2'] !== 7)
      throw new Error('dynamic literal property key');
    return true;
  },
  callback_order_and_abrupt_identity: function () {
    if (String({toString:function(){return 'guard';}}) !== 'guard') throw new Error('conversion guard');
    var trace = '', n = { [Symbol.toPrimitive]: function(hint) { trace += hint+','; return 1000000000000000.25; } };
    if (String(n) !== '1000000000000000.2' || ''+n !== '1000000000000000.2' ||
        `${n}` !== '1000000000000000.2' || Object.keys({[n]:1})[0] !== '1000000000000000.2' ||
        trace !== 'string,default,string,string,') throw new Error('conversion order');
    var marker = {}, bad = { [Symbol.toPrimitive]: function() { trace += 'throw'; throw marker; } }, caught;
    try { String(bad); } catch (error) { caught = error; }
    if (caught !== marker || trace !== 'string,default,string,string,throw') throw new Error('abrupt identity');
    return true;
  }
};
