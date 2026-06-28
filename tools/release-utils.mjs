import { execSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));

export const repoRoot = resolve(scriptDir, "..");
export const coreCargoPath = resolve(repoRoot, "crates/typeship/Cargo.toml");
export const tsRsCargoPath = resolve(
  repoRoot,
  "crates/typeship-ts-rs/Cargo.toml",
);
export const cargoLockPath = resolve(repoRoot, "Cargo.lock");

export function ensureCleanWorktree() {
  const status = exec("git status --porcelain").trim();
  if (status) {
    fail(`Release requires a clean worktree:\n${status}`);
  }
}

export function ensureOnMainBranch() {
  const branch = exec("git branch --show-current").trim();
  if (branch !== "main") {
    fail(`Release must run from main; current branch is ${branch || "<detached>"}`);
  }
}

export function readWorkspaceVersions() {
  return {
    core: readPackageVersion(coreCargoPath),
    adapter: readPackageVersion(tsRsCargoPath),
    adapterDependency: readTypeshipDependencyVersion(tsRsCargoPath),
  };
}

export function assertWorkspaceVersionsSync(versions = readWorkspaceVersions()) {
  if (
    versions.adapter === versions.core &&
    versions.adapterDependency === versions.core
  ) {
    return;
  }

  fail(
    [
      "typeship workspace versions are out of sync:",
      `  crates/typeship: ${versions.core}`,
      `  crates/typeship-ts-rs: ${versions.adapter}`,
      `  typeship-ts-rs dependency on typeship: ${versions.adapterDependency}`,
    ].join("\n"),
  );
}

export function readPackageVersion(path) {
  const text = readFileSync(path, "utf8");
  const match = text.match(/^\[package\][\s\S]*?^version\s*=\s*"([^"]+)"/m);
  if (!match) {
    fail(`Could not find [package] version in ${path}`);
  }
  return match[1];
}

export function writePackageVersion(path, version) {
  const text = readFileSync(path, "utf8");
  const next = text.replace(
    /^(\[package\][\s\S]*?^version\s*=\s*")[^"]+(")/m,
    `$1${version}$2`,
  );
  if (next === text) {
    fail(`Could not update [package] version in ${path}`);
  }
  writeFileSync(path, next, "utf8");
  console.log(`Updated ${relative(path)}`);
}

export function readTypeshipDependencyVersion(path) {
  const text = readFileSync(path, "utf8");
  const match = text.match(
    /^typeship\s*=\s*\{[^}]*version\s*=\s*"([^"]+)"[^}]*\}/m,
  );
  if (!match) {
    fail(`Could not find typeship dependency version in ${path}`);
  }
  return match[1];
}

export function writeTypeshipDependencyVersion(path, version) {
  const text = readFileSync(path, "utf8");
  const next = text.replace(
    /^(typeship\s*=\s*\{[^}]*version\s*=\s*")[^"]+("[^}]*\})/m,
    `$1${version}$2`,
  );
  if (next === text) {
    fail(`Could not update typeship dependency version in ${path}`);
  }
  writeFileSync(path, next, "utf8");
  console.log(`Updated ${relative(path)} dependency version`);
}

export function resolveNextVersion(currentVersion, requested) {
  const current = parseSemver(currentVersion);
  if (!current) {
    return { ok: false, message: `Invalid current version: ${currentVersion}` };
  }

  if (requested === "major") {
    return { ok: true, version: formatSemver(current.major + 1, 0, 0) };
  }
  if (requested === "minor") {
    return { ok: true, version: formatSemver(current.major, current.minor + 1, 0) };
  }
  if (requested === "patch") {
    return {
      ok: true,
      version: formatSemver(current.major, current.minor, current.patch + 1),
    };
  }

  if (!/^\d+\.\d+\.\d+$/.test(requested)) {
    return {
      ok: false,
      message: `Unknown bump type or invalid version: ${requested}`,
      usage: "Usage: npm run release:patch OR node tools/release.mjs [patch|minor|major|x.y.z]",
    };
  }

  const custom = parseSemver(requested);
  return {
    ok: true,
    version: formatSemver(custom.major, custom.minor, custom.patch),
  };
}

export function parseSemver(value) {
  const parts = value.split(".").map(Number);
  if (parts.length !== 3 || parts.some(Number.isNaN)) {
    return null;
  }
  const [major, minor, patch] = parts;
  return { major, minor, patch };
}

export function formatSemver(major, minor, patch) {
  return `${major}.${minor}.${patch}`;
}

export function compareSemver(left, right) {
  for (const part of ["major", "minor", "patch"]) {
    if (left[part] !== right[part]) {
      return left[part] - right[part];
    }
  }
  return 0;
}

export function gitTagExists(tagName) {
  const localTag = exec(`git tag --list ${shellQuote(tagName)}`).trim();
  if (localTag === tagName) {
    return true;
  }

  const remoteTag = exec(
    `git ls-remote --tags origin ${shellQuote(`refs/tags/${tagName}`)}`,
  ).trim();
  return remoteTag.length > 0;
}

export function run(command) {
  execSync(command, { cwd: repoRoot, stdio: "inherit" });
}

export function exec(command) {
  return execSync(command, { cwd: repoRoot, encoding: "utf8" });
}

export function shellQuote(value) {
  return `'${String(value).replaceAll("'", "'\\''")}'`;
}

export function relative(path) {
  return path.startsWith(`${repoRoot}/`) ? path.slice(repoRoot.length + 1) : path;
}

export function fail(message) {
  console.error(message);
  process.exit(1);
}
