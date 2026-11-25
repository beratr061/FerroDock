import { readFileSync } from "node:fs";
import { join } from "node:path";
import { gzipSync } from "node:zlib";

const DIST = new URL("../dist/", import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, "$1");
const LIMIT_KB = 120;

const html = readFileSync(join(DIST, "index.html"), "utf8");
const files = [...new Set([...html.matchAll(/(?:src|href)="\/(assets\/[^"]+\.js)"/g)].map((match) => match[1]))];
if (files.length === 0) {
  console.error("No JavaScript referenced from dist/index.html. Run the build first.");
  process.exit(1);
}

const sizes = files.map((file) => ({ file, gzip: gzipSync(readFileSync(join(DIST, file)), { level: 9 }).length }));
const totalKb = sizes.reduce((sum, entry) => sum + entry.gzip, 0) / 1000;
for (const entry of sizes) {
  console.log(`${(entry.gzip / 1000).toFixed(2).padStart(8)} kB  ${entry.file}`);
}
console.log(`${totalKb.toFixed(2).padStart(8)} kB  initial JavaScript, gzip (limit ${LIMIT_KB} kB)`);
if (totalKb > LIMIT_KB) {
  console.error(`Initial JavaScript is ${(totalKb - LIMIT_KB).toFixed(2)} kB over the budget.`);
  process.exit(1);
}
