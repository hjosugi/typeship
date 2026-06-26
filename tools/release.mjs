import { execSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(scriptDir, "..");

const coreCargoPath = resolve(repoRoot, "crates/typeship/Cargo.toml");
const tsRsCargoPath = resolve(repoRoot, "crates/typeship-ts-rs/Cargo.toml");
const cargoLockPath = resolve(repoRoot, "Cargo.lock");

const bumpType = process.argv[2] || "patch";

ensureCleanWorktree();
ensureOnMainBranch();

const currentVersion = readPackageVersion(coreCargoPath);
const adapterVersion = readPackageVersion(tsRsCargoPath);
const adapterDependencyVersion = readTypeshipDependencyVersion(tsRsCargoPath);

if (adapterVersion !== currentVersion || adapterDependencyVersion !== currentVersion) {
  fail(
    [
      "typeship workspace versions are out of sync:",
      `  crates/typeship: ${currentVersion}`,
      `  crates/typeship-ts-rs: ${adapterVersion}`,
      `  typeship-ts-rs dependency on typeship: ${adapterDependencyVersion}`,
    ].join("\n"),
  );
}

const versionResult = resolveNextVersion(currentVersion, bumpType);
if (!versionResult.ok) {
  fail([versionResult.message, versionResult.usage].filter(Boolean).join("\n"));
}

const newVersion = versionResult.version;
const tagName = `v${newVersion}`;

if (compareSemver(parseSemver(newVersion), parseSemver(currentVersion)) <= 0) {
  fail(`Release version must be greater than ${currentVersion}; got ${newVersion}`);
}

if (gitTagExists(tagName)) {
  fail(`Tag already exists: ${tagName}`);
}

console.log(`Bumping version from ${currentVersion} to ${newVersion}...`);

writePackageVersion(coreCargoPath, newVersion);
writePackageVersion(tsRsCargoPath, newVersion);
writeTypeshipDependencyVersion(tsRsCargoPath, newVersion);

console.log("Updating Cargo.lock...");
run("cargo check --workspace");

console.log("Staging modified files...");
run(
  [
    "git",
    "add",
    shellQuote(coreCargoPath),
    shellQuote(tsRsCargoPath),
    shellQuote(cargoLockPath),
  ].join(" "),
);

const commitMsg = `chore: release ${tagName}`;
console.log(`Committing: ${commitMsg}`);
run(`git commit -m ${shellQuote(commitMsg)}`);

console.log(`Creating tag: ${tagName}`);
run(`git tag -a ${shellQuote(tagName)} -m ${shellQuote(`Release ${tagName}`)}`);

console.log("Pushing commits and tags to GitHub...");
run("git push origin main --follow-tags");

console.log(`\nVersion bumped, tagged, and pushed successfully: ${tagName}`);

function ensureCleanWorktree() {
  const status = execSync("git status --porcelain", {
    cwd: repoRoot,
    encoding: "utf8",
  }).trim();
  if (status) {
    fail(`Release requires a clean worktree:\n${status}`);
  }
}

function ensureOnMainBranch() {
  const branch = execSync("git branch --show-current", {
    cwd: repoRoot,
    encoding: "utf8",
  }).trim();
  if (branch !== "main") {
    fail(`Release must run from main; current branch is ${branch || "<detached>"}`);
  }
}

function readPackageVersion(path) {
  const text = readFileSync(path, "utf8");
  const match = text.match(/^\[package\][\s\S]*?^version\s*=\s*"([^"]+)"/m);
  if (!match) {
    fail(`Could not find [package] version in ${path}`);
  }
  return match[1];
}

function writePackageVersion(path, version) {
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

function writeTypeshipDependencyVersion(path, version) {
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

function resolveNextVersion(currentVersion, requested) {
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

function parseSemver(value) {
  const parts = value.split(".").map(Number);
  if (parts.length !== 3 || parts.some(Number.isNaN)) {
    return null;
  }
  const [major, minor, patch] = parts;
  return { major, minor, patch };
}

function formatSemver(major, minor, patch) {
  return `${major}.${minor}.${patch}`;
}

function compareSemver(left, right) {
  for (const part of ["major", "minor", "patch"]) {
    if (left[part] !== right[part]) {
      return left[part] - right[part];
    }
  }
  return 0;
}

function gitTagExists(tagName) {
  const result = execSync(`git tag --list ${shellQuote(tagName)}`, {
    cwd: repoRoot,
    encoding: "utf8",
  }).trim();
  return result === tagName;
}

function run(command) {
  execSync(command, { cwd: repoRoot, stdio: "inherit" });
}

function shellQuote(value) {
  return `'${String(value).replaceAll("'", "'\\''")}'`;
}

function relative(path) {
  return path.startsWith(`${repoRoot}/`) ? path.slice(repoRoot.length + 1) : path;
}

function fail(message) {
  console.error(message);
  process.exit(1);
}
