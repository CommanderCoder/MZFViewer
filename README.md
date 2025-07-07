# MZFViewer
Sharp MZ computer file viewer

For SP-5025 and SA-5510 Basic

## Features

You can drag a MZF file to the file box (or press the button).  It will detokenise the binary file into the BASIC code. You can do what you like with that code.

## Background

I was after a quick way to view MZF files.  These are files that can be loaded by Sharp MZ computer emulators (MZ80K, MZ80A, MZ700).

After some searching I found https://github.com/tautology0/detokenisers but I wanted a web version.  I've also wanted a project to test out Gemini to help me creating software.  I've been intrigued by Rust and WASM so this seemed like a good project.

A day of sharing files, and writing prompts produced this.

# Gemini and tautology0
. https://gemini.google.com
. https://github.com/tautology0/detokenisers

# Build Instructions

You need [rustup](https://www.rust-lang.org/tools/install) and [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) and [http-server](https://www.npmjs.com/package/http-server).

`wasm-pack build --target web --out-dir public/pkg && cp index.html public/index.html`

`http-server public`