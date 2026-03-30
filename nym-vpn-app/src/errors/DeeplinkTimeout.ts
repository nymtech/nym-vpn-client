/** Thrown when `useDeepLink`’s `startListening` exceeds the given timeout. */
export class DeeplinkTimeout extends Error {
  constructor(message = 'Deeplink timed out') {
    super(message);
    this.name = 'DeeplinkTimeout';
    Object.setPrototypeOf(this, new.target.prototype);
  }
}
