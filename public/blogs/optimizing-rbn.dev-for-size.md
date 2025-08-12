
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

The second thing to note is the `regex_automata` taking up 2.61% of the program.
I need to figure out if I can remove that dep as well.