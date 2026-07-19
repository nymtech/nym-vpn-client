import { expect, test } from '@playwright/test';
import MainPage from '../pages/MainPage';
import NodeListPage from '../pages/NodeList';

test.describe('NodesList', () => {
  let nodeListPage: NodeListPage;

  test.beforeEach(async ({ page }) => {
    nodeListPage = new NodeListPage(page);
    await nodeListPage.goto();
  });

  test('renders nodes list screen correctly', async () => {
    await expect(nodeListPage.searchInput).toBeVisible();
    await expect(nodeListPage.infoButton).toBeVisible();
    await expect(nodeListPage.backButton).toBeVisible();
    await expect(nodeListPage.countSummary).toBeVisible();
    await expect(nodeListPage.randomOption).toBeVisible();
    await expect(nodeListPage.country('FR')).toBeVisible();
    await expect(nodeListPage.country('RU')).toBeVisible();
    await expect(nodeListPage.country('US')).toBeVisible();
  });

  test('renders entry and exit tabs', async () => {
    await expect(nodeListPage.entryTab).toBeVisible();
    await expect(nodeListPage.exitTab).toBeVisible();
  });

  test('switches between entry and exit tabs', async () => {
    await nodeListPage.entryTab.click();
    await expect(nodeListPage.entryTab).toHaveAttribute(
      'aria-selected',
      'true',
    );
    await expect(nodeListPage.exitTab).toHaveAttribute(
      'aria-selected',
      'false',
    );

    await nodeListPage.exitTab.click();
    await expect(nodeListPage.exitTab).toHaveAttribute('aria-selected', 'true');
  });

  test('opens location info modal correctly', async () => {
    await nodeListPage.openInfoModal();

    await expect(nodeListPage.infoModal).toBeVisible();
    await expect(nodeListPage.streamingHeading).toBeVisible();
    await expect(nodeListPage.locationAccuracyHeading).toBeVisible();

    await nodeListPage.closeInfoModal();

    await expect(nodeListPage.infoModal).toHaveCount(0);
  });

  test('filters the list by search term', async () => {
    await expect(nodeListPage.country('US')).toBeVisible();

    await nodeListPage.fillSearchInput('France');

    await expect(nodeListPage.country('FR')).toBeVisible();
    await expect(nodeListPage.country('US')).toHaveCount(0);
  });

  test('shows no results for an unknown search term', async () => {
    await nodeListPage.fillSearchInput('nowhere-at-all');

    await expect(nodeListPage.country('FR')).toHaveCount(0);
    await expect(nodeListPage.country('RU')).toHaveCount(0);
    await expect(nodeListPage.country('US')).toHaveCount(0);
  });

  test('clears search input correctly', async () => {
    await nodeListPage.fillSearchInput('France');
    await expect(nodeListPage.country('US')).toHaveCount(0);

    await nodeListPage.clearSearchInput();

    expect(await nodeListPage.getSearchInputValue()).toBe('');
    await expect(nodeListPage.country('US')).toBeVisible();
  });

  test('expands and collapses country items correctly', async ({ page }) => {
    const expandButton = nodeListPage.expandButtons.nth(1);
    await expect(expandButton).not.toHaveAttribute('aria-expanded', 'true');

    await expandButton.click();

    await expect(expandButton).toHaveAttribute('aria-expanded', 'true');
    // Expanding reveals the country's individual gateways.
    await expect(page.getByText('la porte en bois')).toBeVisible();

    await expandButton.click();

    await expect(expandButton).not.toHaveAttribute('aria-expanded', 'true');
  });

  test('navigates back home', async ({ page }) => {
    // Arrive via home so the back button has history to pop.
    const mainPage = new MainPage(page);
    await mainPage.goto();
    await mainPage.serverRow('Entry').click();
    await expect(page).toHaveURL(/\/node-location/);

    await nodeListPage.backButton.click();

    await expect(page).toHaveURL(/\/home/);
  });
});
