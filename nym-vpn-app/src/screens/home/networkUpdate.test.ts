import assert from 'node:assert/strict';
import test from 'node:test';
import { readFileSync } from 'node:fs';
import {
  blocksNewConnect,
  networkUpdateBodyKey,
  networkUpdateTitleKey,
  showsNetworkUpdateDialog,
} from './networkUpdate.ts';
import type { NetworkCompat } from '../../types/tauri.ts';

function compat(
  partial: Partial<NetworkCompat> & Pick<NetworkCompat, 'appUpdatePolicy'>,
): NetworkCompat {
  return {
    core: true,
    tauri: true,
    ...partial,
  };
}

test('windows outdated dismissible can close and connect', () => {
  const value = compat({ tauri: false, appUpdatePolicy: 'dismissible' });
  assert.equal(showsNetworkUpdateDialog(value, false), true);
  assert.equal(blocksNewConnect(value, false), false);
});

test('outdated required stays and blocks connect', () => {
  const value = compat({ tauri: false, appUpdatePolicy: 'required' });
  assert.equal(showsNetworkUpdateDialog(value, false), true);
  assert.equal(blocksNewConnect(value, false), true);
});

test('current version with required does not prompt or block', () => {
  const value = compat({ appUpdatePolicy: 'required' });
  assert.equal(showsNetworkUpdateDialog(value, false), false);
  assert.equal(blocksNewConnect(value, false), false);
});

test('dev mode skips an outdated required client', () => {
  const value = compat({ tauri: false, appUpdatePolicy: 'required' });
  assert.equal(showsNetworkUpdateDialog(value, true), false);
  assert.equal(blocksNewConnect(value, true), false);
});

test('missing policy is dismissible', () => {
  const value = compat({ core: false, appUpdatePolicy: '' });
  assert.equal(blocksNewConnect(value, false), false);
});

test('unknown policy is dismissible', () => {
  const value = compat({ tauri: false, appUpdatePolicy: 'nope' });
  assert.equal(showsNetworkUpdateDialog(value, false), true);
  assert.equal(blocksNewConnect(value, false), false);
});

test('dismissible copy does not say the app is unsupported', () => {
  assert.equal(networkUpdateTitleKey(false), 'update-dialog.title-available');
  assert.equal(networkUpdateTitleKey(true), 'update-dialog.title');
  assert.equal(networkUpdateBodyKey(false), 'update-dialog.description-available');
  const home = JSON.parse(
    readFileSync(new URL('../../i18n/en/home.json', import.meta.url), 'utf8'),
  ) as { 'update-dialog': Record<string, string> };
  const body = home['update-dialog']['description-available'];
  assert.equal(body.includes('no longer supported'), false);
  assert.match(body, /keep using this version/);
});
