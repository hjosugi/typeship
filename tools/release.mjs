import {
  assertWorkspaceVersionsSync,
  cargoLockPath,
  compareSemver,
  coreCargoPath,
  ensureCleanWorktree,
  ensureOnMainBranch,
  fail,
  gitTagExists,
  parseSemver,
  readWorkspaceVersions,
  resolveNextVersion,
  run,
  shellQuote,
  tsRsCargoPath,
  writePackageVersion,
  writeTypeshipDependencyVersion,
} from "./release-utils.mjs";

const bumpType = process.argv[2] || "patch";

ensureCleanWorktree();
ensureOnMainBranch();

const versions = readWorkspaceVersions();
assertWorkspaceVersionsSync(versions);
const currentVersion = versions.core;

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
  fail(`Tag already exists locally or on origin: ${tagName}`);
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
