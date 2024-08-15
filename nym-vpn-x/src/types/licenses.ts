// types of generated Rust licenses JSON file
export type RustLicensesJson = RustLicenseJson[];

export type RustLicenseJson = {
  name: string;
  version: string;
  authors?: string;
  repository?: string;
  license: string;
  description?: string;
};

// types of generated JS licenses JSON file
export type JsLicensesJson = Record<
  // key is the package name with this structure: `package@semver`
  string,
  {
    licenses?: string | string[];
    repository?: string;
    publisher?: string;
    email?: string;
    copyright?: string;
  }
>;
