// Run automatically by `npm version` (see the "version" script in package.json)
// so tauri.conf.json and Cargo.toml stay in sync with package.json's version,
// which is what gets tagged and built by .github/workflows/release.yml.
import { readFileSync, writeFileSync } from "node:fs";

const { version } = JSON.parse(readFileSync("package.json", "utf8"));

const confPath = "src-tauri/tauri.conf.json";
const conf = JSON.parse(readFileSync(confPath, "utf8"));
conf.version = version;
writeFileSync(confPath, JSON.stringify(conf, null, 2) + "\n");

const cargoPath = "src-tauri/Cargo.toml";
const cargo = readFileSync(cargoPath, "utf8");
writeFileSync(cargoPath, cargo.replace(/^version = ".*"$/m, `version = "${version}"`));

console.log(`Synced version ${version} to ${confPath} and ${cargoPath}`);
