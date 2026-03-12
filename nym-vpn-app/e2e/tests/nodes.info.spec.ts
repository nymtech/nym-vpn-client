import { test, expect } from '@playwright/test';
import NodeListPage from '../pages/NodeList';
import NodeInfoPage from '../pages/NodeInfo';

test.describe('NodeInfo', () => {
  let nodeListPage: NodeListPage;
  let nodeInfoPage: NodeInfoPage;

  test.beforeEach(async ({ page }) => {
    nodeListPage = new NodeListPage(page);
    nodeInfoPage = new NodeInfoPage(page);
    await nodeListPage.gotoEntryServer();
    await nodeListPage.waitForPageLoad();
  });

  test('opens node details screen correctly', async () => {
    await nodeListPage.clickExpandButton(0);
    await nodeListPage.clickNodeDetailsButton(0);

    await expect(nodeInfoPage.SELECTORS.title).toBeVisible();
    await expect(nodeInfoPage.SELECTORS.eixtIPv4Text).toBeVisible();
    await expect(nodeInfoPage.SELECTORS.ASNText).toBeVisible();
    await expect(nodeInfoPage.SELECTORS.ASNNameText).toBeVisible();
    await expect(nodeInfoPage.SELECTORS.advancedPrivacyText).toBeVisible();
    await expect(nodeInfoPage.SELECTORS.streamingContentText).toBeVisible();
    await expect(nodeInfoPage.SELECTORS.antiCensorshipText).toBeVisible();
    await expect(nodeInfoPage.SELECTORS.overallPerformanceText).toBeVisible();
    await expect(nodeInfoPage.SELECTORS.serverLoadText).toBeVisible();
    await expect(nodeInfoPage.SELECTORS.uptimeText).toBeVisible();
    await expect(
      nodeInfoPage.SELECTORS.performanceCalculatedText,
    ).toBeVisible();
    await expect(nodeInfoPage.SELECTORS.nymBuildVersionText).toBeVisible();
    await expect(nodeInfoPage.SELECTORS.identityKeyText).toBeVisible();
    await expect(nodeInfoPage.SELECTORS.copyIdentityKeyButton).toBeVisible();
    await expect(nodeInfoPage.SELECTORS.incorrectInfoLink).toBeVisible();
    await expect(nodeInfoPage.SELECTORS.moreDetailsLink).toBeVisible();
    await expect(nodeInfoPage.SELECTORS.selectServerButton).toBeVisible();
  });

  test('selects server correctly', async ({ page }) => {
    await nodeListPage.clickExpandButton(0);
    await nodeListPage.clickNodeDetailsButton(0);
    await nodeInfoPage.clickSelectServerButton();
    await expect(page.getByText(/__anon__/)).toBeVisible();
  });
});
