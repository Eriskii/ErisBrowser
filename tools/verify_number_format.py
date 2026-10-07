#!/usr/bin/env python3
"""Verify the frozen Number spelling fixtures with exact rational arithmetic.

Find shortest decimal candidates inside each binary64 rounding interval,
then choose nearest and even. This is independent of the proposed midpoint
repair and never uses Python float or binary-to-decimal float formatting.
"""
import csv
import json
from fractions import Fraction
from pathlib import Path


def power10(q):
    return Fraction(10 ** q) if q >= 0 else Fraction(1, 10 ** -q)


def magnitude(bits):
    e = (bits >> 52) & 2047
    m = bits & ((1 << 52) - 1)
    if e:
        m |= 1 << 52
    e = e - 1023 - 52 if e else -1074
    return Fraction(m << e) if e >= 0 else Fraction(m, 1 << -e)


def render(s, q):
    digits = str(s)
    n = len(digits) + q
    if 0 < n <= 21:
        if n >= len(digits):
            return digits + '0' * (n - len(digits))
        return digits[:n] + '.' + digits[n:]
    if -6 < n <= 0:
        return '0.' + '0' * -n + digits
    exponent = n - 1
    mantissa = digits[0] + ('.' + digits[1:] if len(digits) > 1 else '')
    return mantissa + 'e' + ('+' if exponent >= 0 else '') + str(exponent)


def exact_expected(bits):
    sign = '-' if bits >> 63 else ''
    bits &= (1 << 63) - 1
    if bits >= 0x7ff0000000000000:
        return (sign + 'Infinity' if bits == 0x7ff0000000000000 else 'NaN'), 0
    if bits == 0:
        return '0', 0
    x = magnitude(bits)
    low = (magnitude(bits - 1) + x) / 2
    following = Fraction(1 << 1024) if bits == 0x7fefffffffffffff else magnitude(bits + 1)
    high = (x + following) / 2
    inclusive = bits % 2 == 0
    d = len(str(x.numerator)) - len(str(x.denominator))
    while x < power10(d):
        d -= 1
    while x >= power10(d + 1):
        d += 1
    for k in range(1, 18):
        candidates = []
        # The positive interval cannot cross more than one decimal decade.
        for q in range(d - k, d - k + 3):
            scale = power10(q)
            a, b = low / scale, high / scale
            first = (a.numerator + a.denominator - 1) // a.denominator
            last = b.numerator // b.denominator
            if not inclusive and a.denominator == 1:
                first += 1
            if not inclusive and b.denominator == 1:
                last -= 1
            first, last = max(first, 10 ** (k - 1)), min(last, 10 ** k - 1)
            for s in range(first, last + 1):
                if s % 10:
                    candidates.append((abs(s * scale - x), s % 2, s, q))
        if candidates:
            best = min(candidates)
            peers = [c for c in candidates if c[:2] == best[:2]]
            assert len(peers) == 1, (bits, peers)
            return sign + render(best[2], best[3]), k
    raise AssertionError(('no shortest candidate through 17 digits', hex(bits)))


def main():
    source = Path(__file__).resolve().parents[1] / 'tests/conformance/number-format-literals.tsv'
    with source.open(newline='', encoding='utf-8') as stream:
        data = list(csv.DictReader(stream, delimiter='\t'))
    assert len(data) == 74
    assert len({row['name'] for row in data}) == 74
    checks = []
    for row in data:
        actual, digits = exact_expected(int(row['bits_hex'], 16))
        assert actual == row['expected'], (row['name'], actual, row['expected'])
        checks.append({'name': row['name'], 'bits_hex': row['bits_hex'],
                       'literal': row['expected'], 'minimum_digits': digits})
    print(json.dumps({'status': 'clear', 'method': 'exact rational interval enumeration',
                      'rows': len(checks), 'checks': checks}, indent=2))


if __name__ == '__main__':
    main()
