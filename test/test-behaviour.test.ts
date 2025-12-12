#!/usr/bin/env -S bun run --

import { expect, test } from "bun:test";
import { Readable } from "node:stream";
import { Path } from "path-class";
import { PrintableShellCommand } from "printable-shell-command";

const EXAMPLES = new Path("./examples/");

let compilePromise: Promise<void> | undefined;
async function cmd(
  args: ConstructorParameters<typeof PrintableShellCommand>[1],
) {
  // biome-ignore lint/suspicious/noAssignInExpressions: Caching pattern.
  await (compilePromise ??= new PrintableShellCommand("cargo", [
    "build",
    "--release",
  ]).shellOut());
  return new PrintableShellCommand("./target/release/folderify", args);
}

async function shellOut(
  args: ConstructorParameters<typeof PrintableShellCommand>[1],
) {
  return (await cmd(args)).shellOut();
}

test("Help flag", async () => {
  await shellOut(["--help"]);
});

test("Generate icon file", async () => {
  await shellOut([EXAMPLES.join("src/apple.png")]);
  expect(await EXAMPLES.join("src/apple.icns").existsAsFile()).toBe(true);
  expect(await EXAMPLES.join("src/apple.iconset/").existsAsDir()).toBe(true);
  for (const pngSuffix of [
    "512x512@2x",
    "256x256@2x",
    "512x512",
    "128x128@2x",
    "256x256",
    "128x128",
    "16x16@2x",
    "16x16",
    "32x32@2x",
    "32x32",
  ]) {
    expect(
      await EXAMPLES.join(
        `src/apple.iconset/icon_${pngSuffix}.png`,
      ).existsAsFile(),
    ).toBe(true);
  }
});

test("Assign folder icon", async () => {
  const tempDir = await Path.makeTempDir();
  await shellOut([EXAMPLES.join("src/apple.png"), tempDir]);
  expect(await tempDir.join("Icon\r").exists()).toBe(true);
});

test("Assign folder icon using Rez", async () => {
  const tempDir = await Path.makeTempDir();
  await shellOut([
    ["--set-icon-using", "Rez"],
    EXAMPLES.join("src/apple.png"),
    tempDir,
  ]);
  expect(await tempDir.join("Icon\r").exists()).toBe(true);
});

test("Test that `--verbose` is accepted.", async () => {
  await shellOut(["--verbose", EXAMPLES.join("src/apple.png")]);
});

test("Test that `--no-trim` is accepted.", async () => {
  await shellOut(["--no-trim", EXAMPLES.join("src/apple.png")]);
});

test("Test that `--color-scheme auto` is accepted.", async () => {
  await shellOut([["--color-scheme", "auto"], EXAMPLES.join("src/apple.png")]);
});

test("Test that `--color-scheme light` is accepted.", async () => {
  await shellOut([["--color-scheme", "light"], EXAMPLES.join("src/apple.png")]);
});

test("Test that `--color-scheme dark` is accepted.", async () => {
  await shellOut([["--color-scheme", "dark"], EXAMPLES.join("src/apple.png")]);
});

test("Test that `--no-progress` is accepted.", async () => {
  await shellOut(["--no-progress", EXAMPLES.join("src/apple.png")]);
});

test("Test that `--badge alias` is accepted.", async () => {
  await shellOut([["--badge", "alias"], EXAMPLES.join("src/apple.png")]);
});

test("Test that `--output-icns …` works.", async () => {
  await shellOut([
    ["--output-icns", EXAMPLES.join("./src/folder_outline_custom_path_1.icns")],
    EXAMPLES.join("src/apple.png"),
  ]);
  expect(
    await EXAMPLES.join("./src/folder_outline_custom_path_1.icns").exists(),
  ).toBe(true);
  expect(await EXAMPLES.join("./src/folder_outline.icns").exists()).toBe(false);
  expect(await EXAMPLES.join("./src/folder_outline.iconset").exists()).toBe(
    false,
  );
});

test("Test that `--output-iconset …` works.", async () => {
  await shellOut([
    [
      "--output-iconset",
      EXAMPLES.join("./src/folder_outline_custom_path_2.iconset"),
    ],
    EXAMPLES.join("src/apple.png"),
  ]);
  expect(
    await EXAMPLES.join("./src/folder_outline_custom_path_2.iconset").exists(),
  ).toBe(true);
  expect(await EXAMPLES.join("./src/folder_outline.icns").exists()).toBe(false);
  expect(await EXAMPLES.join("./src/folder_outline.iconset").exists()).toBe(
    false,
  );
});

test("Test that `--output-icns …` and `--output-iconset …` can be used together.", async () => {
  await shellOut([
    ["--output-icns", EXAMPLES.join("./src/folder_outline_custom_path_3.icns")],
    [
      "--output-iconset",
      EXAMPLES.join("./src/folder_outline_custom_path_4.iconset"),
    ],
    EXAMPLES.join("src/apple.png"),
  ]);
  expect(
    await EXAMPLES.join("./src/folder_outline_custom_path_3.icns").exists(),
  ).toBe(true);
  expect(
    await EXAMPLES.join("./src/folder_outline_custom_path_4.iconset").exists(),
  ).toBe(true);
  expect(await EXAMPLES.join("./src/folder_outline.icns").exists()).toBe(false);
  expect(await EXAMPLES.join("./src/folder_outline.iconset").exists()).toBe(
    false,
  );
});

for (const macOSVersion of ["10.5", "10.8", "10.15"]) {
  test(`Test that known macOS ${macOSVersion} is rejected`, async () => {
    expect(
      await (async () => {
        const { stderr } = (
          await cmd([["--macOS", macOSVersion], EXAMPLES.join("src/apple.png")])
        ).spawn({ stdio: ["ignore", "ignore", "pipe"] });
        return new Response(Readable.from(stderr)).text();
      })(),
    ).toMatch(
      "Error: OS X / macOS 10 was specified. This is no longer supported by folderify v3.",
    );
  });
}

for (const macOSVersion of ["10.16", "99.0"]) {
  test(`Test that known macOS ${macOSVersion} is accepted with a warning`, async () => {
    expect(
      await (async () => {
        const { stderr } = (
          await cmd([["--macOS", macOSVersion], EXAMPLES.join("src/apple.png")])
        ).spawn({ stdio: ["ignore", "ignore", "pipe"] });
        return new Response(Readable.from(stderr)).text();
      })(),
    ).toMatch("Warning: Unknown macOS version specified.");
  });
}

for (const macOSVersion of ["11.0", "12.1", "14.2.1", "26"]) {
  test(`Test that known macOS ${macOSVersion} is accepted without a warning`, async () => {
    expect(
      await (async () => {
        const { stderr } = (
          await cmd([["--macOS", macOSVersion], EXAMPLES.join("src/apple.png")])
        )
          .print()
          .spawn({ stdio: ["ignore", "ignore", "pipe"] });
        return new Response(Readable.from(stderr)).text();
      })(),
    ).not.toMatch("Warning: Unknown macOS version specified.");
  });
}
