import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, writeFileSync } from "node:fs";

const VERSION_RE = /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/;
const RELEASE_BRANCH = "main";
const VERSION_FILES = [
  "Cargo.toml",
  "Cargo.lock",
  "frontend/package.json",
];

function run(command, args, options = {}) {
  return execFileSync(command, args, {
    encoding: "utf8",
    stdio: options.stdio ?? "pipe",
    ...options,
  });
}

function gitOutput(args) {
  return run("git", args).trim();
}

function ensureCleanWorktree() {
  const status = gitOutput(["status", "--porcelain"]);
  if (status) {
    throw new Error("working tree must be clean before releasing a version");
  }
}

function ensureReleaseBranch() {
  const branch = gitOutput(["branch", "--show-current"]);
  if (branch !== RELEASE_BRANCH) {
    throw new Error(
      `release versions must be created on ${RELEASE_BRANCH}; current branch is ${branch || "detached HEAD"}`
    );
  }
}

function ensureNoExistingTag(version) {
  const tag = `v${version}`;
  try {
    gitOutput(["rev-parse", "-q", "--verify", `refs/tags/${tag}`]);
    throw new Error(`tag ${tag} already exists`);
  } catch (error) {
    if (error.status === 1) {
      return;
    }
    throw error;
  }
}

function readPackageVersion() {
  const packageJson = JSON.parse(readFileSync("frontend/package.json", "utf8"));
  return packageJson.version;
}

function readCargoVersion() {
  const cargoToml = readFileSync("Cargo.toml", "utf8");
  const match = cargoToml.match(/^\[package\][\s\S]*?^version\s*=\s*"([^"]+)"/m);
  if (!match) {
    throw new Error("failed to read Cargo.toml package version");
  }
  return match[1];
}

function ensureCurrentVersionsMatch() {
  const packageVersion = readPackageVersion();
  const cargoVersion = readCargoVersion();
  if (packageVersion !== cargoVersion) {
    throw new Error(
      `version mismatch: Cargo.toml is ${cargoVersion}, frontend/package.json is ${packageVersion}`
    );
  }
  return packageVersion;
}

function writePackageVersion(version) {
  const packagePath = "frontend/package.json";
  const packageJson = JSON.parse(readFileSync(packagePath, "utf8"));
  packageJson.version = version;
  writeFileSync(packagePath, `${JSON.stringify(packageJson, null, 2)}\n`);
}

function writeCargoVersion(version) {
  const cargoPath = "Cargo.toml";
  const cargoToml = readFileSync(cargoPath, "utf8");
  const nextCargoToml = cargoToml.replace(
    /(^\[package\][\s\S]*?^version\s*=\s*)"[^"]+"/m,
    `$1"${version}"`
  );

  if (nextCargoToml === cargoToml) {
    throw new Error("failed to update Cargo.toml package version");
  }

  writeFileSync(cargoPath, nextCargoToml);
}

function updateCargoLock() {
  run("cargo", ["metadata", "--format-version", "1"], { stdio: "ignore" });
}

function verifyRelease() {
  run("just", ["check"], { stdio: "inherit" });
}

function commitAndTag(version) {
  const existingFiles = VERSION_FILES.filter((file) => existsSync(file));
  run("git", ["add", ...existingFiles], { stdio: "inherit" });

  const staged = gitOutput(["diff", "--cached", "--name-only"]);
  if (!staged) {
    throw new Error("no version changes to commit");
  }

  run(
    "git",
    ["commit", "-m", `chore(release): bump version to ${version}`],
    { stdio: "inherit" }
  );
  run("git", ["tag", `v${version}`], { stdio: "inherit" });
}

function normalizeVersion(input) {
  return input.trim().replace(/^[vV]/, "");
}

function main() {
  const versionArg = process.argv[2];
  if (!versionArg) {
    throw new Error("usage: just release-version VERSION");
  }

  const version = normalizeVersion(versionArg);
  if (!VERSION_RE.test(version)) {
    throw new Error(`invalid semver version: ${versionArg}`);
  }

  ensureCleanWorktree();
  ensureReleaseBranch();
  ensureNoExistingTag(version);

  const oldVersion = ensureCurrentVersionsMatch();
  if (oldVersion === version) {
    throw new Error(`version is already ${version}`);
  }

  console.log(`Current version: ${oldVersion}`);
  console.log(`New version: ${version}`);

  writePackageVersion(version);
  writeCargoVersion(version);
  updateCargoLock();
  verifyRelease();
  commitAndTag(version);

  console.log(
    `Release version prepared. Push with:\ngit push && git push origin v${version}`
  );
}

try {
  main();
} catch (error) {
  console.error(error.message);
  process.exit(1);
}
