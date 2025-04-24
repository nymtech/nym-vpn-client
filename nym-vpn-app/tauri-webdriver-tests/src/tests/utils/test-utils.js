async function waitForInteractable(element, timeout = 5000, errorMessage = '') {
  const customMessage =
    errorMessage || `Element not interactable after ${timeout}ms`;

  try {
    await element.waitForDisplayed({ timeout });
    await element.waitForEnabled({ timeout });
    return true;
  } catch (error) {
    console.error(`${customMessage}: ${error.message}`);
    throw new Error(customMessage);
  }
}

async function waitAndClick(element, timeout = 5000) {
  await waitForInteractable(
    element,
    timeout,
    `Could not click element within ${timeout}ms`,
  );
  await element.click();
}

async function waitAndGetText(element, timeout = 5000) {
  await element.waitForDisplayed({ timeout });
  return element.getText();
}

async function waitAndSetValue(element, value, timeout = 5000) {
  await waitForInteractable(
    element,
    timeout,
    `Could not set value on element within ${timeout}ms`,
  );
  await element.clearValue();
  await element.setValue(value);
}

async function retry(actionFn, maxAttempts = 3, interval = 1000) {
  let lastError;

  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    try {
      return await actionFn();
    } catch (error) {
      console.warn(
        `Attempt ${attempt}/${maxAttempts} failed: ${error.message}`,
      );
      lastError = error;

      if (attempt < maxAttempts) {
        await new Promise((resolve) => setTimeout(resolve, interval));
      }
    }
  }

  throw new Error(
    `All ${maxAttempts} attempts failed. Last error: ${lastError.message}`,
  );
}

async function waitForTextToContain(element, text, timeout = 5000) {
  await element.waitForDisplayed({ timeout });

  return browser.waitUntil(
    async () => {
      const actualText = await element.getText();
      return actualText.includes(text);
    },
    {
      timeout,
      timeoutMsg: `Expected element to contain "${text}" within ${timeout}ms`,
    },
  );
}

async function waitForTextToEqual(element, text, timeout = 5000) {
  await element.waitForDisplayed({ timeout });

  return browser.waitUntil(
    async () => {
      const actualText = await element.getText();
      return actualText === text;
    },
    {
      timeout,
      timeoutMsg: `Expected element to equal "${text}" within ${timeout}ms`,
    },
  );
}

async function waitForElementToDisappear(element, timeout = 5000) {
  return browser.waitUntil(
    async () => {
      try {
        return !(await element.isExisting());
      } catch (e) {
        // Element already removed from DOM
        return true;
      }
    },
    {
      timeout,
      timeoutMsg: `Expected element to disappear within ${timeout}ms`,
    },
  );
}

async function waitForAttributeToEqual(
  element,
  attribute,
  value,
  timeout = 5000,
) {
  return browser.waitUntil(
    async () => {
      const attrValue = await element.getAttribute(attribute);
      return attrValue === value;
    },
    {
      timeout,
      timeoutMsg: `Expected attribute "${attribute}" to equal "${value}" within ${timeout}ms`,
    },
  );
}

async function takeScreenshot(name) {
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
  const filename = `${name}-${timestamp}.png`;
  return browser.saveScreenshot(`./screenshots/${filename}`);
}

async function waitForTauriRerender(ms = 500) {
  await browser.pause(ms);
}

async function isSelected(element) {
  try {
    const checked = await element.getAttribute('aria-checked');
    return checked === 'true';
  } catch (error) {
    try {
      const dataChecked = await element.getAttribute('data-checked');
      return dataChecked === 'true' || dataChecked === '';
    } catch (err) {
      return false;
    }
  }
}

module.exports = {
  waitForInteractable,
  waitAndClick,
  waitAndGetText,
  waitAndSetValue,
  retry,
  waitForTextToContain,
  waitForTextToEqual,
  waitForElementToDisappear,
  waitForAttributeToEqual,
  takeScreenshot,
  waitForTauriRerender,
  isSelected,
};
