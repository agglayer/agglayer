"use strict";

const fs = require("node:fs");
const path = require("node:path");
const { createHash } = require("node:crypto");

const root = path.resolve(__dirname, "..");
const lock = JSON.parse(fs.readFileSync(path.join(root, "package-lock.json"), "utf8"));
const packages = Object.entries(lock.packages)
  .filter(([location, metadata]) => location.startsWith("node_modules/") && metadata.dev !== true)
  .sort(([left], [right]) => left.localeCompare(right));
const sections = packages.map(([location, metadata]) => {
  const directory = path.join(root, location);
  const name = location.split("node_modules/").at(-1);
  const licenseFile = fs.readdirSync(directory).find((file) => /^licen[cs]e(?:\..*)?$/i.test(file));
  let license;
  let note = "";
  if (licenseFile) license = fs.readFileSync(path.join(directory, licenseFile), "utf8").trim();
  else if (name === "standardwebhooks" && metadata.version === "1.0.0") {
    const rawLicense = fs.readFileSync(path.join(__dirname, "licenses/standardwebhooks-1.0.0.txt"), "utf8");
    const hash = createHash("sha256").update(rawLicense).digest("hex");
    if (hash !== "5ec8c7b26b64d881a6706617bed25c049f97f2f35de034c756de8546fd6dbe27")
      throw new Error("The pinned standardwebhooks@1.0.0 license text changed.");
    license = rawLicense.trim();
    note = "\nLicense copied from libraries/LICENSE at the upstream v1.0.0 tag.\n";
  } else throw new Error(`No license text found for ${name}@${metadata.version}`);
  return `${"=".repeat(80)}\n${name}@${metadata.version}\nSPDX metadata: ${metadata.license ?? "unknown"}${note}\n${license}`;
});
const notice = `THIRD-PARTY SOFTWARE NOTICES

Generated from package-lock.json by npm run notices.
This file contains the license text shipped with every locked production dependency.

${sections.join("\n\n")}\n`;
fs.writeFileSync(path.join(root, "THIRD_PARTY_NOTICES.txt"), notice);
