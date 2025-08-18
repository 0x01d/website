---
title: "Optimizing rbn.dev's wasm binary. - Part 2: removing the bloat"
tags: ["wasm", "rust", "configs"]
date: 2025-8-8
---

# Part 2: Removing the bloat.

So the game plan for this blog is removing the biggest bloat from the codebase.
The two biggest offenders in the codebase are Syntect and Serde.

Let's start with serde and see what we can do:

```
[user@local ratzilla_app]$ cargo tree --invert serde
serde v1.0.219
├── bincode v1.3.3
│   ├── gloo-worker v0.5.0
│   │   └── gloo v0.11.0
│   │       └── rasm v0.1.0 (/home/user/dev/0x01d/rasm/ratzilla_app)
│   └── syntect v5.2.0
│       └── rasm v0.1.0 (/home/user/dev/0x01d/rasm/ratzilla_app)
├── chrono v0.4.41
│   └── rasm v0.1.0 (/home/user/dev/0x01d/rasm/ratzilla_app)
├── gloo-console v0.3.0
│   └── gloo v0.11.0 (*)
├── gloo-history v0.2.2
│   └── gloo v0.11.0 (*)
├── gloo-net v0.5.0
│   └── gloo v0.11.0 (*)
├── gloo-net v0.6.0
│   └── rasm v0.1.0 (/home/user/dev/0x01d/rasm/ratzilla_app)
├── gloo-storage v0.3.0
│   └── gloo v0.11.0 (*)
├── gloo-utils v0.2.0
│   ├── gloo v0.11.0 (*)
│   ├── gloo-console v0.3.0 (*)
│   ├── gloo-history v0.2.2 (*)
│   ├── gloo-net v0.5.0 (*)
│   ├── gloo-net v0.6.0 (*)
│   ├── gloo-storage v0.3.0 (*)
│   └── gloo-worker v0.5.0 (*)
├── gloo-worker v0.5.0 (*)
├── plist v1.7.4
│   └── syntect v5.2.0 (*)
├── rasm v0.1.0 (/home/user/dev/0x01d/rasm/ratzilla_app)
├── serde-wasm-bindgen v0.6.5
│   └── gloo-history v0.2.2 (*)
├── serde_json v1.0.142
│   ├── gloo-net v0.5.0 (*)
│   ├── gloo-net v0.6.0 (*)
│   ├── gloo-storage v0.3.0 (*)
│   ├── gloo-utils v0.2.0 (*)
│   ├── rasm v0.1.0 (/home/user/dev/0x01d/rasm/ratzilla_app)
│   └── syntect v5.2.0 (*)
├── serde_urlencoded v0.7.1
│   └── gloo-history v0.2.2 (*)
└── syntect v5.2.0 (*)
```

So the easiest win looks like removing `gloo`, which is just a bunch of wrappers 
around `wasm-bindgen`. But then there is still `Syntect`, the code highlighter.
The problem I have is that I don't know of any other lib for code-highlighting 
that is better.

Then one has to ask himself, what is the benefit of code-highlighting on the 
client. Since I already use a blog generation step to parse all the blogs and
tags.

So I decided to tokenize using syntect in the build step and parse the tokens 
manually on the client inside the wasm. I feel stupid that I didn't think of
this before implementing a syntax parser on the frontend -lol sometimes you need
to think before acting ;).

I prompted claude to remove the dependencies on syntect and serde, it did a 
remarkably good job at removing serde and syntect from the deps. But in the 
meantime it broke some functionality of the tags system. Let me fix that quickly
and keep going with removing bloateru.
