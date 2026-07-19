import { expect, test } from '@playwright/test';
import MainPage from '../pages/MainPage';
import NodeInfoPage from '../pages/NodeInfo';
import NodeListPage from '../pages/NodeList';

test.describe('NodeInfo', () => {
  let nodeListPage: NodeListPage;
  let nodeInfoPage: NodeInfoPage;

  test.beforeEach(async ({ page }) => {
    nodeListPage = new NodeListPage(page);
    nodeInfoPage = new NodeInfoPage(page);
    await nodeListPage.goto();
    // Expand a country to reveal its gateways, then open the first one.
    await nodeListPage.clickExpandButton(1);
    await nodeListPage.clickNodeDetailsButton(0);
    await expect(page).toHaveURL(/\/node-details/);
  });

  test('opens node details screen correctly', async () => {
    await expect(nodeInfoPage.title).toBeVisible();

    // Node features
    await expect(nodeInfoPage.advancedPrivacyLabel).toBeVisible();
    await expect(nodeInfoPage.streamingLabel).toBeVisible();
    await expect(nodeInfoPage.antiCensorshipLabel).toBeVisible();

    // Performance metrics
    await expect(nodeInfoPage.overallPerformanceLabel).toBeVisible();
    await expect(nodeInfoPage.serverLoadLabel).toBeVisible();
    await expect(nodeInfoPage.uptimeLabel).toBeVisible();
    await expect(nodeInfoPage.performanceCalculatedNote).toBeVisible();

    // Connection details
    await expect(nodeInfoPage.exitIpv4Label).toBeVisible();
    await expect(nodeInfoPage.asnLabel).toBeVisible();
    await expect(nodeInfoPage.asnNameLabel).toBeVisible();

    // Build information
    await expect(nodeInfoPage.buildVersionLabel).toBeVisible();
    await expect(nodeInfoPage.identityKeyLabel).toBeVisible();
    await expect(nodeInfoPage.incorrectInfoLink).toBeVisible();
    await expect(nodeInfoPage.selectServerButton).toBeVisible();
  });

  test('renders the mocked node metrics', async () => {
    await expect(nodeInfoPage.overallPerformanceValue).toBeVisible();
    await expect(nodeInfoPage.serverLoadValue).toBeVisible();
    await expect(nodeInfoPage.uptimeValue.first()).toBeVisible();
    await expect(nodeInfoPage.asnValue).toBeVisible();
    await expect(nodeInfoPage.asnNameValue).toBeVisible();
  });

  test('hints at enabling QUIC when it is off', async () => {
    await expect(nodeInfoPage.antiCensorshipValue).toBeVisible();
    await expect(nodeInfoPage.enableQuicHint).toBeVisible();
  });

  test('selects server correctly', async ({ page }) => {
    await nodeInfoPage.clickSelectServerButton();

    // Selecting returns to home with the picked gateway shown on the row.
    await expect(page).toHaveURL(/\/home/);
    const mainPage = new MainPage(page);
    await expect(mainPage.serverRow('Exit')).not.toContainText('Random');
  });

  test('navigates back to the node list', async ({ page }) => {
    await nodeInfoPage.backButton.click();

    await expect(page).toHaveURL(/\/node-location/);
  });
});
