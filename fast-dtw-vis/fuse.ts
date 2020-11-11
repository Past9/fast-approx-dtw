import { FuseBox, CSSPlugin, Sparky, CopyPlugin } from "fuse-box"
import { spawn } from "child_process"

const PORT = 4445
const OUTPUT_DIR = "out"
const ASSETS = ["*.jpg", "*.png", "*.jpeg", "*.gif", "*.svg"]

Sparky.task("copy-html", () => {
  return Sparky.src("src/app/index.html").dest(`${OUTPUT_DIR}/$name`)
})

Sparky.task("default", ["copy-html"], () => {
  const fuse = FuseBox.init({
    homeDir: "src",
    output: `${OUTPUT_DIR}/$name.js`,
    target: "electron",
    log: false,
    cache: true,
    sourceMaps: true,
    tsConfig: "tsconfig.json",
  })

  fuse.dev({ port: PORT, httpServer: false })

  const mainBundle = fuse
    .bundle("main")
    .target("server")
    .instructions("> [app/main.ts]")

  mainBundle.watch()

  const rendererBundle = fuse
    .bundle("renderer")
    .instructions("> [app/index.tsx] +fuse-box-css")
    .plugin(CSSPlugin())
    .plugin(
      CopyPlugin({
        useDefault: false,
        files: ASSETS,
        dest: "assets",
        resolve: "assets/",
      })
    )

  rendererBundle.watch()
  rendererBundle.hmr()

  return fuse.run().then(() => {
    spawn("node", [`${__dirname}/node_modules/electron/cli.js`, __dirname], {
      stdio: "inherit",
    }).on("exit", (code) => {
      console.log(`electron process exited with code ${code}`)
      process.exit(code)
    })
  })
})
