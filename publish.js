const { execSync } = require("child_process");
const { readFileSync, writeFileSync } = require("fs");

const oldVersion = execSync("pnpm pkg get version", {
  cwd: "./frontend",
})
  .toString()
  .trim();
console.log("Current version:", oldVersion.replace(/"/g, ""));

// receive version arg
process.stdin.setEncoding("utf8");
console.log("Input new version:");
process.stdin.on("data", function (data) {
  var version = data.trim();
  if (!version) {
    console.log("version is required");
    console.log("Input new version:");
    return;
  }
  process.stdin.pause();
  console.log("publishing version: " + version);

  console.log("update package.json version");
  console.log(
    execSync("pnpm version --no-git-tag-version " + version, {
      cwd: "./frontend",
    }).toString()
  );

  console.log("update cargo.toml version\n");
  const cargo_path = "./Cargo.toml";
  const cargo_lines = readFileSync(cargo_path, "utf8").split("\n");
  for (let i = 0; i < cargo_lines.length; i++) {
    if (cargo_lines[i].startsWith("version")) {
      cargo_lines[i] = `version = "${version}"`;
      break;
    }
  }
  writeFileSync(cargo_path, cargo_lines.join("\n"));

  console.log("git commit and tag");
  console.log(
    execSync(
      `git add . && git commit -m "Scrcpy Mask v${version}" && git tag v${version}`
    ).toString()
  );

  console.log(
    "Pleash push commit and tag to github manually:\ngit push && git push --tags"
  );
});
