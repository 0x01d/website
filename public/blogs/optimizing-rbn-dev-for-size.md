
# Optimizing rbn.dev's wasm binary

At the time of writing this, the website is in it's first beta version. The big
problem was all the bloat in my code. The binary has ballooned to 2.9Mb. Not fun.

So first step was to install `twiggy` and analyze where the bloat is coming from.
On first run I noticed massive `.rodata` section and the function names taking up
loads of space, even though I compiled in release mode. 

```bash
§c§[user@local§/§ rasm]$ twiggy top dist/rasm-572497de7343eb4_bg.wasm | §c§head§/§
 §c§Shallow§/§ Bytes │ Shallow % │ Item
§c§───────────────┼───────────┼─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────§/§
       §c§1024490§/§ ┊    34.35% ┊ data segment "§s§.rodata§/§"
        §c§280760§/§ ┊     9.41% ┊ "§s§function names§/§" subsection
         §c§77975§/§ ┊     2.61% ┊ regex_automata::meta::strategy::new::h39624e35bccabf79
         §c§16240§/§ ┊     0.54% ┊ pulldown_cmark::firstpass::FirstPass::parse_block::hb2a6949987e47973
         §c§14091§/§ ┊     0.47% ┊ rasm::app::tools::window_scanner::DeepExtensionScanner::run_deep_scan::h78a6f21ae37a5d2b
         §c§13787§/§ ┊     0.46% ┊ <regex_syntax::hir::translate::TranslatorI as regex_syntax::ast::visitor::Visitor>::visit_post::h8ddeb8ef2542ff44
         §c§13698§/§ ┊     0.46% ┊ regex_automata::nfa::thompson::compiler::Compiler::c::h88f7778e36c6a21b
         §c§12681§/§ ┊     0.43% ┊ regex_automata::hybrid::search::find_fwd::h7167a838aa9d1335
```

The second thing to note is the `regex_automata` is the biggest bloat in the bin
but I want to use syntect for code highlighting, so let's keep it for now. 

So the first step was optimizing the build for size. Setting the following settings
has managed to shave off 1mb off the bin. Woohoo! Performance wise nothing has 
changed.

```toml
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
the following configuration in `.config.toml`:

```toml
[target.wasm32-unknown-unknown]
rustflags = [
    "-C", "panic=abort",
    "-C", "opt-level=z",
    #"-C", "lto=true",
    "--remap-path-prefix=/home/user=~",
]

[unstable]
# Requires the rust-src component. `rustup +nightly component add rust-src`
build-std = ["std", "panic_abort"]
build-std-features = ["panic_immediate_abort", "optimize_for_size"]
```
This will build core and std, instead of using the default version and it'll
remove the panic handler, completely.

Curiously when deploying to Netlify, it didn't do jack.. I forgot to update my
build script to use the nightly toolchain. build-std only works on nightly ;)

Here's the build script, for reference:

```bash
§c§#§/§§c§!/bin/bash§/§§c§
§/§§t§set§/§ -e

§t§echo§/§ "§s§🦀 Setting up Rust environment...§/§"

§c§#§/§§c§ Install Rust if not present§/§§c§
§/§§k§if§/§ ! §t§command§/§ -v cargo &> /dev/null; §k§then§/§
    §t§echo§/§ "§s§Installing Rust...§/§"
    §c§curl§/§ https://sh.rustup.rs -sSf | §c§sh§/§ -s -- -y --default-toolchain stable
    §t§source§/§ $HOME/.cargo/env
§k§else§/§
    §t§echo§/§ "§s§Rust already installed§/§"
§k§fi§/§

§t§echo§/§ "§s§Rustup default stable§/§"
§c§rustup§/§ default nightly

§c§#§/§§c§ Add wasm target§/§§c§
§/§§t§echo§/§ "§s§Adding WASM target...§/§"
§c§rustup§/§ target add wasm32-unknown-unknown

§c§#§/§§c§ Install trunk if not present§/§§c§
§/§§k§if§/§ ! §t§command§/§ -v trunk &> /dev/null; §k§then§/§
    §t§echo§/§ "§s§Installing trunk...§/§"
    §c§cargo§/§ install trunk
§k§else§/§
    §t§echo§/§ "§s§Trunk already installed§/§"
§k§fi§/§

§c§#§/§§c§ Optional: Install wasm-opt for smaller binaries§/§§c§
§/§§c§#§/§§c§ cargo install wasm-opt§/§§c§
§/§§t§cd§/§ ratzilla_app
§t§echo§/§ "§s§📦 Building WASM application...§/§"
§c§trunk§/§ build --release

§t§echo§/§ "§s§✅ Build complete!§/§"
```

This moved my size down to 1740636 bytes, great stuff. But still over a Mb, so 
un-acceptable for a simple blog. I do have the MutationObserver on there, it is
a tool to detected which plugins have access to DOM and what globals are loaded
in the window, it's rather useless to be honest, I just wanted to print cool
stuff to my web terminal, hehe.. So let's use the unix philosophy and have one 
tool that does one thing good. In my case, a blog that loads a TUI-themed blog 
with blazing speeds. If I want to add a tool to the website I'll add them later 
as a seperate wasm.

> Note: While writing this blog I fixed scrolling functionality which upped the
> size to 1.9Mb again.

Yay! Removing the Useless tool shrank size to 1671881 bytes. Let's check which
dependencies are unused to see if we can shave off a bit more. For this I can 
use `cargo-machete`!

```bash
§c§[user@local§/§ ratzilla_app]$ cargo machete
§c§Analyzing§/§ dependencies of crates in this directory...
§c§cargo-machete§/§ found the following unused dependencies in this directory:
§c§rasm§/§ -- ./Cargo.toml:
	§c§color-eyre§/§
	§c§js-sys§/§
```

So that only found two, I know for a fact there is more we can do, so let's go 
the manual way and comment out, compile, repeat. This didn't seem to change the 
bytes amount. So instead let's focus on code. Since this blog post is all over 
the place already. Let's keep that one for part 2.
