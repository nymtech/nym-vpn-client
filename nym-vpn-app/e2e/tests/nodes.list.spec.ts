import { test, expect } from '@playwright/test';
import NodeListPage from '../pages/NodeList';

test.describe('NodesList', () => {
  let nodeListPage: NodeListPage;

  test.beforeEach(async ({ page }) => {
    nodeListPage = new NodeListPage(page);
    await nodeListPage.gotoEntryServer();
    await nodeListPage.waitForPageLoad();
  });

  test('renders nodes list screen correctly', async () => {
    await expect(nodeListPage.SELECTORS.title).toBeVisible();
    await expect(nodeListPage.SELECTORS.searchInput).toBeVisible();
    await expect(nodeListPage.SELECTORS.infoButton).toBeVisible();
    await expect(nodeListPage.SELECTORS.backButton).toBeVisible();
    await expect(nodeListPage.SELECTORS.countryItems.first()).toBeVisible();
  });

  test('opens entry node info modal correctly', async () => {
    await nodeListPage.clickInfoButton();
    await expect(nodeListPage.SELECTORS.nodeInfoModal).toBeVisible();
    await expect(nodeListPage.SELECTORS.quicProtocolText).toBeVisible();
    await expect(nodeListPage.SELECTORS.locationAccuracyText).toBeVisible();
    await expect(nodeListPage.SELECTORS.closeModalButton).toBeVisible();

    await nodeListPage.closeInfoModal();

    await expect(nodeListPage.SELECTORS.nodeInfoModal).not.toBeVisible();
    await expect(nodeListPage.SELECTORS.quicProtocolText).not.toBeVisible();
    await expect(nodeListPage.SELECTORS.locationAccuracyText).not.toBeVisible();
    await expect(nodeListPage.SELECTORS.closeModalButton).not.toBeVisible();
  });

  test('opens exit node info modal correctly', async () => {
    await nodeListPage.gotoExitServer();
    await nodeListPage.waitForPageLoad();
    await nodeListPage.clickInfoButton();

    await expect(nodeListPage.SELECTORS.nodeInfoModal).toBeVisible();
    await expect(nodeListPage.SELECTORS.streamingText).toBeVisible();
    await expect(nodeListPage.SELECTORS.locationAccuracyText).toBeVisible();
    await expect(nodeListPage.SELECTORS.closeModalButton).toBeVisible();

    await nodeListPage.closeInfoModal();

    await expect(nodeListPage.SELECTORS.nodeInfoModal).not.toBeVisible();
    await expect(nodeListPage.SELECTORS.streamingText).not.toBeVisible();
    await expect(nodeListPage.SELECTORS.locationAccuracyText).not.toBeVisible();
    await expect(nodeListPage.SELECTORS.closeModalButton).not.toBeVisible();
  });

  test('clears search input correctly', async () => {
    await nodeListPage.fillSearchInput('Russia');
    expect(await nodeListPage.getSearchInputValue()).toBe('Russia');
    await nodeListPage.clickClearSearchButton();
    expect(await nodeListPage.getSearchInputValue()).toBe('');
  });

  test('expands and collapses country items correctly', async ({ page }) => {
    const nodeSelector = page.getByRole('button', { name: '__anon__ Sydney' });
    await nodeListPage.clickExpandButton(0);
    await expect(nodeSelector).toBeVisible();
    await nodeListPage.clickExpandButton(0);
    await expect(nodeSelector).not.toBeVisible();
  });
});
