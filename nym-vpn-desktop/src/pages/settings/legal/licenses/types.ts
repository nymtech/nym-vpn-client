export type Dependency = {
  name: string;
  version?: string;
  licenses: string[];
  licenseTexts: string[];
  repository?: string;
  authors: string[];
  copyright?: string;
};

export type DependencyByNames = {
  [key: string]: Dependency;
};

// types of generated Rust licenses JSON file
export type RustLicensesJson = {
  overview: RustOverview[];
  licenses: RustLicenseJson[];
  crates: RustPackage[];
};

type RustOverview = {
  count: number;
  name: string;
  id: string;
  indices: number[];
  text: string;
};

type RustLicenseJson = {
  name: string;
  id: string;
  text: string;
  source_path?: string;
  used_by: { crate: Crate }[];
};

type RustPackage = {
  package: Crate;
  license: string;
};

type Crate = {
  name: string;
  version: string;
  authors: string[];
  id: string;
  source?: string;
  description?: string;
  license?: string;
  manifest_path: string;
  categories: string[];
  keywords: string[];
  readme?: string;
  repository?: string;
  homepage?: string;
};

// types of generated JS licenses JSON file
export type JsLicensesJson = {
  // key is the package name with this structure: `package@semver`
  [key: string]: {
    licenses?: string | string[];
    repository?: string;
    publisher?: string;
    email?: string;
    licenseText?: string;
    copyright?: string;
  };
};
