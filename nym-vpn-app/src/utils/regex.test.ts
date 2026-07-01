import { describe, expect, it } from 'vitest';
import { ipv4Regex, ipv6Regex } from './regex';

describe('ipv4Regex', () => {
  it.each([
    '0.0.0.0',
    '127.0.0.1',
    '192.168.1.1',
    '255.255.255.255',
    '8.8.8.8',
  ])('matches valid address %s', (addr) => {
    expect(ipv4Regex.test(addr)).toBe(true);
  });

  it.each([
    '256.0.0.1', // octet out of range
    '1.2.3', // too few octets
    '1.2.3.4.5', // too many octets
    '192.168.1.', // trailing dot
    'abc.def.ghi.jkl', // non-numeric
    '', // empty
  ])('rejects invalid address %s', (addr) => {
    expect(ipv4Regex.test(addr)).toBe(false);
  });
});

describe('ipv6Regex', () => {
  it.each([
    '::1',
    '::',
    'fe80::1',
    '2001:0db8:85a3:0000:0000:8a2e:0370:7334',
    '2001:db8::8a2e:370:7334',
  ])('matches valid address %s', (addr) => {
    expect(ipv6Regex.test(addr)).toBe(true);
  });

  it.each([
    'gggg::1', // invalid hex
    '12345::1', // group too long
    '1.2.3.4', // ipv4, not ipv6
    '', // empty
  ])('rejects invalid address %s', (addr) => {
    expect(ipv6Regex.test(addr)).toBe(false);
  });
});
