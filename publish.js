import { execSync } from "child_process";
import { readFileSync, writeFileSync } from "fs";

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
    execSync("pnpm version --no-git-tag-version " + version).toString()
  );

  console.log("update cargo.toml version\n");
  const lines = readFileSync("./src-tauri/Cargo.toml", "utf8").split("\n");
  for (let i = 0; i < lines.length; i++) {
    if (lines[i].startsWith("version")) {
      lines[i] = `version = "${version}"`;
      break;
    }
  }
  writeFileSync("./src-tauri/Cargo.toml", lines.join("\n"));

  console.log("git commit and tag");
  console.log(execSync(`git add . && git commit -m "Scrcpy Mask v${version}" && git tag v${version}`).toString());

  console.log("Pleash push to github or cancel manually");
});
