import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const tag = process.argv[2] ?? process.env.GITHUB_REF_NAME ?? "";
const version = tag.replace(/^v/, "");

if (!/^\d+\.\d+\.\d+$/.test(version)) {
  fail(`Release tag must look like vX.Y.Z; got ${tag || "<empty>"}`);
}

const scriptDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(scriptDir, "..");
const coreCargoPath = resolve(repoRoot, "crates/typeship/Cargo.toml");
const tsRsCargoPath = resolve(repoRoot, "crates/typeship-ts-rs/Cargo.toml");

const coreVersion = readPackageVersion(coreCargoPath);
const tsRsVersion = readPackageVersion(tsRsCargoPath);
const dependencyVersion = readTypeshipDependencyVersion(tsRsCargoPath);

if (
  coreVersion !== version ||
  tsRsVersion !== version ||
  dependencyVersion !== version
) {
  fail(
    [
      `Release tag ${tag} does not match workspace versions:`,
      `  crates/typeship: ${coreVersion}`,
      `  crates/typeship-ts-rs: ${tsRsVersion}`,
      `  typeship-ts-rs dependency on typeship: ${dependencyVersion}`,
    ].join("\n"),
  );
}

console.log(`Release tag ${tag} matches workspace version ${version}.`);

function readPackageVersion(path) {
  const text = readFileSync(path, "utf8");
  const match = text.match(/^\[package\][\s\S]*?^version\s*=\s*"([^"]+)"/m);
  if (!match) {
    fail(`Could not find [package] version in ${path}`);
  }
  return match[1];
}

function readTypeshipDependencyVersion(path) {
  const text = readFileSync(path, "utf8");
  const match = text.match(
    /^typeship\s*=\s*\{[^}]*version\s*=\s*"([^"]+)"[^}]*\}/m,
  );
  if (!match) {
    fail(`Could not find typeship dependency version in ${path}`);
  }
  return match[1];
}

function fail(message) {
  console.error(message);
  process.exit(1);
}
