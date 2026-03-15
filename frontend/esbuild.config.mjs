import esbuild from "esbuild";

const watch = process.argv.includes("--watch");

const ctx = await esbuild.context({
  entryPoints: ["src/main.ts"],
  bundle: true,
  outfile: "dist/wiregraph.js",
  format: "esm",
  platform: "browser",
  target: "es2022",
  sourcemap: true,
  minify: !watch,
});

if (watch) {
  await ctx.watch();
  console.log("watching for changes...");
} else {
  await ctx.rebuild();
  await ctx.dispose();
  console.log("built dist/wiregraph.js");
}
