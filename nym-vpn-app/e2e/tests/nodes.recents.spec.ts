import { expect, test } from '@playwright/test';
import MainPage from '../pages/MainPage';
import NodeListPage from '../pages/NodeList';

/**
 * The recents view of the node location screen.
 *
 * The mocked daemon (`src/dev/setup.ts`) serves recents per mode and hop: dVPN
 * mode has recents for both hops, mixnet has none. The screen opens on the Exit
 * tab, so that hop's recents are what the default assertions describe.
 */
test.describe('NodesRecents', () => {
  let nodeListPage: NodeListPage;

  test.beforeEach(async ({ page }) => {
    nodeListPage = new NodeListPage(page);
    await nodeListPage.goto();
  });

  test('renders the recents tab in the view toggle', async () => {
    await expect(nodeListPage.viewToggle).toBeVisible();
    await expect(nodeListPage.recentsView).toBeVisible();
    await expect(nodeListPage.allView).toHaveAttribute('aria-pressed', 'true');
    await expect(nodeListPage.recentsView).toHaveAttribute(
      'aria-pressed',
      'false',
    );
  });

  test('switches to the recents view', async () => {
    await nodeListPage.showRecents();

    await expect(nodeListPage.recentsView).toHaveAttribute(
      'aria-pressed',
      'true',
    );
    await expect(nodeListPage.recentsList).toBeVisible();
  });

  test('lists recent servers most-recent-first', async () => {
    await nodeListPage.showRecents();

    await expect(nodeListPage.recentRows).toHaveCount(2);
    expect(await nodeListPage.recentNames()).toEqual([
      'Mikhaïl Boulgakov 😼',
      'High speed gateway',
    ]);
  });

  test('shows the full location on a recent server row', async ({ page }) => {
    await nodeListPage.showRecents();

    // Recents are ungrouped, so each row carries its own country context.
    await expect(page.getByText('Elektrozavodsk, Russia')).toBeVisible();
    await expect(
      page.getByText('New York, New York, United States'),
    ).toBeVisible();
  });

  test('tracks the selected view per hop', async () => {
    await nodeListPage.showRecents();
    await expect(nodeListPage.recentsView).toHaveAttribute(
      'aria-pressed',
      'true',
    );

    await nodeListPage.entryTab.click();

    // Each hop keeps its own view, so Entry is still on the full list.
    await expect(nodeListPage.allView).toHaveAttribute('aria-pressed', 'true');
    await expect(nodeListPage.recentsList).toHaveCount(0);
  });

  test('lists the recents of the selected hop', async () => {
    await nodeListPage.entryTab.click();
    await nodeListPage.showRecents();

    await expect(nodeListPage.recentRows).toHaveCount(3);
    expect(await nodeListPage.recentNames()).toEqual([
      'High speed gateway',
      'la porte en bois',
      'Mikhaïl Boulgakov 😼',
    ]);
  });

  test('filters recents by search term', async () => {
    await nodeListPage.showRecents();
    await nodeListPage.fillSearchInput('New York');

    await expect(nodeListPage.recentRows).toHaveCount(1);
    expect(await nodeListPage.recentNames()).toEqual(['High speed gateway']);
  });

  test('shows no results for an unknown search term', async () => {
    await nodeListPage.showRecents();
    await nodeListPage.fillSearchInput('nowhere-at-all');

    await expect(nodeListPage.recentsNoResults).toBeVisible();
    await expect(nodeListPage.recentRows).toHaveCount(0);
    // A search matching nothing must not read as "no recents".
    await expect(nodeListPage.recentsEmpty).toHaveCount(0);
  });

  test('restores the list after clearing the search', async () => {
    await nodeListPage.showRecents();
    await nodeListPage.fillSearchInput('nowhere-at-all');
    await expect(nodeListPage.recentRows).toHaveCount(0);

    await nodeListPage.clearSearchInput();

    await expect(nodeListPage.recentRows).toHaveCount(2);
  });

  test('shows the empty state when the mode has no recents', async ({
    page,
  }) => {
    const mainPage = new MainPage(page);
    await mainPage.goto();
    await mainPage.mixnetMode.click();
    await expect(mainPage.mixnetMode).toHaveAttribute('aria-pressed', 'true');

    // Reached through the UI rather than a reload: the mocked backend does not
    // persist the mode, so reloading would drop back to dVPN.
    await mainPage.serverRow('Exit').click();
    await expect(page).toHaveURL(/\/node-location/);
    await nodeListPage.showRecents();

    await expect(nodeListPage.recentsEmpty).toBeVisible();
    await expect(nodeListPage.recentsList).toHaveCount(0);
  });

  test('keeps the recents view when navigating to node details and back', async ({
    page,
  }) => {
    await nodeListPage.showRecents();
    await nodeListPage.clickNodeDetailsButton(0);
    await expect(page).toHaveURL(/\/node-details/);

    await nodeListPage.backButton.click();

    await expect(page).toHaveURL(/\/node-location/);
    await expect(nodeListPage.recentsView).toHaveAttribute(
      'aria-pressed',
      'true',
    );
    await expect(nodeListPage.recentsList).toBeVisible();
  });
});
