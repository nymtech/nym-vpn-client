import {
  Dependency,
  DependencyByNames,
  JsLicensesJson,
  RustLicensesJson,
} from './types';

// These files are generated by `npm run gen:licenses` command
// and are located in the `public` directory
const LicensesJs = '/licenses-js.json';
const LicensesRust = '/licenses-rust.json';

export async function getRustLicenses(): Promise<Dependency[] | undefined> {
  let json: RustLicensesJson;
  try {
    const response = await fetch(LicensesRust);
    json = await response.json();
  } catch (e) {
    console.warn('Failed to fetch Rust licenses data', e);
    return;
  }

  let list: Dependency[] = [];
  try {
    const crates = json.licenses.reduce<DependencyByNames>(
      (acc, { name: licenseName, text: licenseText, used_by }) => {
        used_by.forEach(({ crate }) => {
          const key = `${crate.name}@${crate.version}`;
          if (acc[key]) {
            if (!acc[key].licenses?.includes(licenseName)) {
              acc[key].licenses?.push(licenseName);
              acc[key].licenseTexts?.push(licenseText);
            }
          } else {
            acc[key] = {
              ...crate,
              licenses: [licenseName],
              licenseTexts: [licenseText],
            };
          }
        });
        return acc;
      },
      {},
    );
    list = Object.values(crates).map((crate) => {
      return {
        ...crate,
        authors: crate.authors,
      };
    });
  } catch (e) {
    console.warn('Failed to parse Rust licenses data', e);
    return;
  }

  return list;
}

export async function getJsLicenses(): Promise<Dependency[] | undefined> {
  let json: JsLicensesJson;
  try {
    const response = await fetch(LicensesJs);
    json = await response.json();
  } catch (e) {
    console.warn('Failed to fetch Js licenses data', e);
    return;
  }

  let list: Dependency[] = [];
  try {
    list = Object.entries(json).map(([name, info]) => {
      let licenses: string[] = [];
      if (info.licenses) {
        if (Array.isArray(info.licenses)) {
          licenses = [...info.licenses];
        } else {
          licenses = [info.licenses];
        }
      }
      // package name is formatted as `name@semver`
      const components = name.split('@');
      const version = components.pop() || '0.0.0';

      return {
        ...info,
        name: components.join('@'),
        version,
        authors: info.publisher ? [info.publisher] : [],
        licenses,
        licenseTexts: info.licenseText ? [info.licenseText] : [],
      };
    });
  } catch (e) {
    console.warn('Failed to parse Js licenses data', e);
    return;
  }

  return list;
}
