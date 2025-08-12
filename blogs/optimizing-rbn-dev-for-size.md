---
title: "Optimizing rbn.dev's wasm binary."
tags: ["wasm", "rust"]
date: 2025-8-8
---

# Optimizing rbn.dev's wasm binary

At the time of writing this, the website is in it's first beta version. The big
problem was all the bloat in my code. The binary has ballooned to 2.9Mb. Not fun.

So first step was to install `twiggy` and analyze where the bloat is coming from.
On first run I noticed massive `.rodata` section and the function names taking up
loads of space, even though I compiled in release mode. 

```
[user@local rasm]$ twiggy top dist/rasm-572497de7343eb4_bg.wasm | head
 Shallow Bytes │ Shallow % │ Item
───────────────┼───────────┼─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
       1024490 ┊    34.35% ┊ data segment ".rodata"
        280760 ┊     9.41% ┊ "function names" subsection
         77975 ┊     2.61% ┊ regex_automata::meta::strategy::new::h39624e35bccabf79
         16240 ┊     0.54% ┊ pulldown_cmark::firstpass::FirstPass::parse_block::hb2a6949987e47973
         14091 ┊     0.47% ┊ rasm::app::tools::window_scanner::DeepExtensionScanner::run_deep_scan::h78a6f21ae37a5d2b
         13787 ┊     0.46% ┊ <regex_syntax::hir::translate::TranslatorI as regex_syntax::ast::visitor::Visitor>::visit_post::h8ddeb8ef2542ff44
         13698 ┊     0.46% ┊ regex_automata::nfa::thompson::compiler::Compiler::c::h88f7778e36c6a21b
         12681 ┊     0.43% ┊ regex_automata::hybrid::search::find_fwd::h7167a838aa9d1335
```

The second thing to note is the `regex_automata` is the biggest bloat in the bin
but I want to use syntect for code highlighting, so let's keep it for now. 

So the first step was optimizing the build for size. Setting the following settings
has managed to shave off 1mb off the bin. Woohoo! Performance wise nothing has 
changed.

```
[profile.release]
opt-level = "z"          # Optimize for size
lto = true              # Link-time optimization
codegen-units = 1       # Single codegen unit
panic = "abort"         # Reduce panic handling code
strip = true            # Strip symbols
```
This shaved a Mb of the bin.

The next mission is getting the data segment down. The first thing I noticed is
that it has a bunch of file-paths from my local machine in there:

```
[user@local rasm]$ wasm-objdump -s -j Data dist/rasm-e8cd0bb5005b3fd_bg.wasm
01d5daf: f82f 686f 6d65 2f75 7365 722f 2e72 7573  ./home/user/.rus
01d5dbf: 7475 702f 746f 6f6c 6368 6169 6e73 2f73  tup/toolchains/s
01d5dcf: 7461 626c 652d 7838 365f 3634 2d75 6e6b  table-x86_64-unk
01d5ddf: 6e6f 776e 2d6c 696e 7578 2d67 6e75 2f6c  nown-linux-gnu/l
01d5def: 6962 2f72 7573 746c 6962 2f73 7263 2f72  ib/rustlib/src/r
01d5dff: 7573 742f 6c69 6272 6172 792f 616c 6c6f  ust/library/allo
01d5e0f: 632f 7372 632f 626f 7865 642f 6974 6572  c/src/boxed/iter
01d5e1f: 2e72 73f6 c61d 0072 0000 0090 0000 002e  .rs....r........
01d5e2f: 0000 0054 7269 6564 2074 6f20 7368 7269  ...Tried to shri
01d5e3f: 6e6b 2074 6f20 6120 6c61 7267 6572 2063  nk to a larger c
01d5e4f: 6170 6163 6974 7978 c71d 0024 0000 002f  apacityx...$.../
01d5e5f: 686f 6d65 2f75 7365 722f 2e72 7573 7475  home/user/.rustu
01d5e6f: 702f 746f 6f6c 6368 6169 6e73 2f73 7461  p/toolchains/sta
01d5e7f: 626c 652d 7838 365f 3634 2d75 6e6b 6e6f  ble-x86_64-unkno
01d5e8f: 776e 2d6c 696e 7578 2d67 6e75 2f6c 6962  wn-linux-gnu/lib
01d5e9f: 2f72 7573 746c 6962 2f73 7263 2f72 7573  /rustlib/src/rus
01d5eaf: 742f 6c69 6272 6172 792f 616c 6c6f 632f  t/library/alloc/
01d5ebf: 7372 632f 7261 775f 7665 632e 7273 00a4  src/raw_vec.rs..
```

So I went for the nuclear option and removed the panic handler completely with
the following configuration:
```toml
[unstable]
# Requires the rust-src component. `rustup +nightly component add rust-src`
build-std = ["std", "panic_abort"]
build-std-features = ["panic_immediate_abort", "optimize_for_size"]
```

This moved my size down to 1740636 bytes, great stuff.

Curiously when adding the following configs to optimize my size even further, it 
grew to 1743583 bytes.
```
[profile.release.build-override]
opt-level = "z"
codegen-units = 1

[profile.release.package."*"]  # All dependencies including std
opt-level = "z"
strip = true
```

So let's remove those again. And start looking at code changes.
