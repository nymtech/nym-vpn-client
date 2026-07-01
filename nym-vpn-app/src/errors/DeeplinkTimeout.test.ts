import { describe, expect, it } from 'vitest';
import { DeeplinkTimeout } from './DeeplinkTimeout';

describe('DeeplinkTimeout', () => {
  it('is an Error instance with the correct name', () => {
    const err = new DeeplinkTimeout();
    expect(err).toBeInstanceOf(Error);
    expect(err).toBeInstanceOf(DeeplinkTimeout);
    expect(err.name).toBe('DeeplinkTimeout');
  });

  it('uses the default message when none is given', () => {
    expect(new DeeplinkTimeout().message).toBe('Deeplink timed out');
  });

  it('preserves a custom message', () => {
    expect(new DeeplinkTimeout('boom').message).toBe('boom');
  });

  it('is catchable as its own type after being thrown', () => {
    try {
      throw new DeeplinkTimeout('x');
    } catch (e) {
      expect(e instanceof DeeplinkTimeout).toBe(true);
    }
  });
});
