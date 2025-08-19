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
meantime it broke some functionality of the tags system. After fixing that, my
total binary size ended up to: **388612 Bytes** good stuff.. 
> After checking I noticed claude didn't do a good job at the frontend side, 
> going to have to fix some bugs.

> Note: when building on netlify, it still shows up as 0.5 Mb, let's keep going
> and investigate later..

Next up, removing gloo. I know I said it would be an easy win, but it's, going
to be harder then expected, since the gloo makes our life a whole lot easier.

I really don't want to f with the manual implementation of the popstate listener
```rust
use gloo::events::EventListener;

let popstate_listener = EventListener::new(&window, "popstate", move |event| {
        popstate_clone.borrow_mut().handle_popstate();
});
```

So I did some checking and I can **fully remove serde** by only removing 
`gloo-net`, which is an easy win at last :). 

With original deps:
```
[user@local ratzilla_app]$ cargo tree --invert serde
serde v1.0.219
├── gloo-net v0.5.0
    │   └── gloo v0.11.0
│       └── rbn-dev v0.1.0 (/home/user/dev/0x01d/rasm/ratzilla_app)
    ├── gloo-net v0.6.0
│   └── rbn-dev v0.1.0 (/home/user/dev/0x01d/rasm/ratzilla_app)
    ├── gloo-utils v0.2.0
    │   ├── gloo-net v0.5.0 (*)
│   └── gloo-net v0.6.0 (*)
    └── serde_json v1.0.142
    ├── gloo-net v0.5.0 (*)
    ├── gloo-net v0.6.0 (*)
└── gloo-utils v0.2.0 (*)
```
With only: 
`gloo = { version = "0.11.0", default-features = false, features = ["events"] }`

```
    [user@local ratzilla_app]$ cargo tree --invert serde
    warning: nothing to print.

    To find dependencies that require specific target platforms, try to use option `--target all` first, and then narrow your search scope accordingly.
```

And we're down to 379704 bytes, might not seem much, but any byte
counts. I think we can push it further, so let's run twiggy again ;).

```
[user@local ratzilla_app]$ twiggy top ../dist/rbn-dev-6528e1b1d2dbcfcb_bg.wasm
 Shallow Bytes │ Shallow % │ Item
───────────────┼───────────┼─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
        115170 ┊    30.33% ┊ data[0]
         13908 ┊     3.66% ┊ code[0]
         12667 ┊     3.34% ┊ code[1]
         11686 ┊     3.08% ┊ code[2]
         11303 ┊     2.98% ┊ code[3]
          7418 ┊     1.95% ┊ code[4]
          6670 ┊     1.76% ┊ code[5]
          5789 ┊     1.52% ┊ code[6]
          3961 ┊     1.04% ┊ code[7]
          3603 ┊     0.95% ┊ code[8]
          3466 ┊     0.91% ┊ code[9]
          3057 ┊     0.81% ┊ code[10]
          2842 ┊     0.75% ┊ code[11]
          2569 ┊     0.68% ┊ code[12]
          2392 ┊     0.63% ┊ code[13]
```

My data section is still 115K, so let's inspect once again. It contains a bunch
of data, which I believe will compress well, so not going to bother there, but
it also contains a bunch of format strings, that are problematic. Like the ones
below for example:

```
0045fe7: 466f 7241 6c6c e288 8046 6f75 7269 6572  ForAll...Fourier
0045ff7: 7472 66e2 84b1 4673 6372 474a 6379 d083  trf...FscrGJcy..
0046007: 4754 3e47 616d 6d61 ce93 4761 6d6d 6164  GT>Gamma..Gammad
0046017: cf9c 4762 7265 7665 c49e 4763 6564 696c  ..Gbreve..Gcedil
0046027: c4a2 4763 6972 63c4 9c47 6379 d093 4764  ..Gcirc..Gcy..Gd
0046037: 6f74 c4a0 4766 72f0 9d94 8a47 67e2 8b99  ot..Gfr....Gg...
0046047: 476f 7066 f09d 94be 4772 6561 7465 7245  Gopf....GreaterE
0046057: 7175 616c e289 a547 7265 6174 6572 4571  qual...GreaterEq
0046067: 7561 6c4c 6573 73e2 8b9b 4772 6561 7465  ualLess...Greate
0046077: 7246 756c 6c45 7175 616c e289 a747 7265  rFullEqual...Gre
0046087: 6174 6572 4772 6561 7465 72e2 aaa2 4772  aterGreater...Gr
0046097: 6561 7465 724c 6573 73e2 89b7 4772 6561  eaterLess...Grea
00460a7: 7465 7253 6c61 6e74 4571 7561 6ce2 a9be  terSlantEqual...
00460b7: 4772 6561 7465 7254 696c 6465 e289 b347  GreaterTilde...G
00460c7: 7363 72f0 9d92 a247 74e2 89ab 4841 5244  scr....Gt...HARD
00460d7: 6379 d0aa 4861 6365 6bcb 8748 6174 5e48  cy..Hacek..Hat^H
00460e7: 6369 7263 c4a4 4866 72e2 848c 4869 6c62  circ..Hfr...Hilb
00460f7: 6572 7453 7061 6365 e284 8b48 6f70 66e2  ertSpace...Hopf.
0046107: 848d 486f 7269 7a6f 6e74 616c 4c69 6e65  ..HorizontalLine
0046117: e294 8048 7363 7248 7374 726f 6bc4 a648  ...HscrHstrok..H
0046127: 756d 7044 6f77 6e48 756d 7048 756d 7045  umpDownHumpHumpE
0046137: 7175 616c e289 8f49 4563 79d0 9549 4a6c  qual...IEcy..IJl
0046147: 6967 c4b2 494f 6379 d081 4961 6375 7465  ig..IOcy..Iacute
0046157: c38d 4963 6972 63c3 8e49 6379 d098 4964  ..Icirc..Icy..Id
0046167: 6f74 c4b0 4966 72e2 8491 4967 7261 7665  ot..Ifr...Igrave
0046177: c38c 496d 496d 6163 72c4 aa49 6d61 6769  ..ImImacr..Imagi
0046187: 6e61 7279 49e2 8588 496d 706c 6965 7349  naryI...ImpliesI
0046197: 6e74 e288 ac49 6e74 6567 7261 6ce2 88ab  nt...Integral...
00461a7: 496e 7465 7273 6563 7469 6f6e e28b 8249  Intersection...I
00461b7: 6e76 6973 6962 6c65 436f 6d6d 61e2 81a3  nvisibleComma...
00461c7: 496e 7669 7369 626c 6554 696d 6573 e281  InvisibleTimes..
00461d7: a249 6f67 6f6e c4ae 496f 7066 f09d 9580  .Iogon..Iopf....
00461e7: 496f 7461 ce99 4973 6372 e284 9049 7469  Iota..Iscr...Iti
00461f7: 6c64 65c4 a849 756b 6379 d086 4975 6d6c  lde..Iukcy..Iuml
0046207: c38f 4a63 6972 63c4 b44a 6379 d099 4a66  ..Jcirc..Jcy..Jf
0046217: 72f0 9d94 8d4a 6f70 66f0 9d95 814a 7363  r....Jopf....Jsc
0046227: 72f0 9d92 a54a 7365 7263 79d0 884a 756b  r....Jsercy..Juk
0046237: 6379 d084 4b48 6379 d0a5 4b4a 6379 d08c  cy..KHcy..KJcy..
0046247: 4b61 7070 61ce 9a4b 6365 6469 6cc4 b64b  Kappa..Kcedil..K
0046257: 6379 d09a 4b66 72f0 9d94 8e4b 6f70 66f0  cy..Kfr....Kopf.
0046267: 9d95 824b 7363 72f0 9d92 a64c 4a63 79d0  ...Kscr....LJcy.
0046277: 894c 543c 4c61 6375 7465 c4b9 4c61 6d62  .LT<Lacute..Lamb
0046287: 6461 ce9b 4c61 6e67 e29f aa4c 6170 6c61  da..Lang...Lapla
0046297: 6365 7472 66e2 8492 4c61 7272 e286 9e4c  cetrf...Larr...L
00462a7: 6361 726f 6ec4 bd4c 6365 6469 6cc4 bb4c  caron..Lcedil..L
00462b7: 6379 d09b 4c65 6674 416e 676c 6542 7261  cy..LeftAngleBra
00462c7: 636b 6574 e29f a84c 6566 7441 7272 6f77  cket...LeftArrow
00462d7: e286 904c 6566 7441 7272 6f77 4261 72e2  ...LeftArrowBar.
00462e7: 87a4 4c65 6674 4172 726f 7752 6967 6874  ..LeftArrowRight
00462f7: 4172 726f 77e2 8786 4c65 6674 4365 696c  Arrow...LeftCeil
0046307: 696e 67e2 8c88 4c65 6674 446f 7562 6c65  ing...LeftDouble
0046317: 4272 6163 6b65 74e2 9fa6 4c65 6674 446f  Bracket...LeftDo
0046327: 776e 5465 6556 6563 746f 72e2 a5a1 4c65  wnTeeVector...Le
0046337: 6674 446f 776e 5665 6374 6f72 e287 834c  ftDownVector...L
0046347: 6566 7444 6f77 6e56 6563 746f 7242 6172  eftDownVectorBar
0046357: e2a5 994c 6566 7446 6c6f 6f72 e28c 8a4c  ...LeftFloor...L
0046367: 6566 7452 6967 6874 4172 726f 77e2 8694  eftRightArrow...
0046377: 4c65 6674 5269 6768 7456 6563 746f 72e2  LeftRightVector.
0046387: a58e 4c65 6674 5465 65e2 8aa3 4c65 6674  ..LeftTee...Left
0046397: 5465 6541 7272 6f77 e286 a44c 6566 7454  TeeArrow...LeftT
00463a7: 6565 5665 6374 6f72 e2a5 9a4c 6566 7454  eeVector...LeftT
00463b7: 7269 616e 676c 65e2 8ab2 4c65 6674 5472  riangle...LeftTr
00463c7: 6961 6e67 6c65 4261 72e2 a78f 4c65 6674  iangleBar...Left
00463d7: 5472 6961 6e67 6c65 4571 7561 6ce2 8ab4  TriangleEqual...
00463e7: 4c65 6674 5570 446f 776e 5665 6374 6f72  LeftUpDownVector
00463f7: e2a5 914c 6566 7455 7054 6565 5665 6374  ...LeftUpTeeVect
0046407: 6f72 e2a5 a04c 6566 7455 7056 6563 746f  or...LeftUpVecto
0046417: 72e2 86bf 4c65 6674 5570 5665 6374 6f72  r...LeftUpVector
0046427: 4261 72e2 a598 4c65 6674 5665 6374 6f72  Bar...LeftVector
0046437: e286 bc4c 6566 7456 6563 746f 7242 6172  ...LeftVectorBar
0046447: e2a5 924c 6566 7461 7272 6f77 4c65 6674  ...LeftarrowLeft
0046457: 7269 6768 7461 7272 6f77 4c65 7373 4571  rightarrowLessEq
0046467: 7561 6c47 7265 6174 6572 e28b 9a4c 6573  ualGreater...Les
0046477: 7346 756c 6c45 7175 616c e289 a64c 6573  sFullEqual...Les
0046487: 7347 7265 6174 6572 e289 b64c 6573 734c  sGreater...LessL
0046497: 6573 73e2 aaa1 4c65 7373 536c 616e 7445  ess...LessSlantE
00464a7: 7175 616c e2a9 bd4c 6573 7354 696c 6465  qual...LessTilde
00464b7: e289 b24c 6672 f09d 948f 4c6c e28b 984c  ...Lfr....Ll...L
00464c7: 6c65 6674 6172 726f 77e2 879a 4c6d 6964  leftarrow...Lmid
00464d7: 6f74 c4bf 4c6f 6e67 4c65 6674 4172 726f  ot..LongLeftArro
00464e7: 77e2 9fb5 4c6f 6e67 4c65 6674 5269 6768  w...LongLeftRigh
00464f7: 7441 7272 6f77 e29f b74c 6f6e 6752 6967  tArrow...LongRig
0046507: 6874 4172 726f 77e2 9fb6 4c6f 6e67 6c65  htArrow...Longle
0046517: 6674 6172 726f 774c 6f6e 676c 6566 7472  ftarrowLongleftr
0046527: 6967 6874 6172 726f 774c 6f6e 6772 6967  ightarrowLongrig
0046537: 6874 6172 726f 774c 6f70 66f0 9d95 834c  htarrowLopf....L
0046547: 6f77 6572 4c65 6674 4172 726f 77e2 8699  owerLeftArrow...
0046557: 4c6f 7765 7252 6967 6874 4172 726f 77e2  LowerRightArrow.
0046567: 8698 4c73 6372 4c73 68e2 86b0 4c73 7472  ..LscrLsh...Lstr
0046577: 6f6b c581 4c74 e289 aa4d 6170 e2a4 854d  ok..Lt...Map...M
0046587: 6379 d09c 4d65 6469 756d 5370 6163 65e2  cy..MediumSpace.
0046597: 819f 4d65 6c6c 696e 7472 66e2 84b3 4d66  ..Mellintrf...Mf
00465a7: 72f0 9d94 904d 696e 7573 506c 7573 e288  r....MinusPlus..
00465b7: 934d 6f70 66f0 9d95 844d 7363 724d 75ce  .Mopf....MscrMu.
00465c7: 9c4e 4a63 79d0 8a4e 6163 7574 65c5 834e  .NJcy..Nacute..N
00465d7: 6361 726f 6ec5 874e 6365 6469 6cc5 854e  caron..Ncedil..N
00465e7: 6379 d09d 4e65 6761 7469 7665 4d65 6469  cy..NegativeMedi
00465f7: 756d 5370 6163 654e 6567 6174 6976 6554  umSpaceNegativeT
0046607: 6869 636b 5370 6163 654e 6567 6174 6976  hickSpaceNegativ
0046617: 6554 6869 6e53 7061 6365 4e65 6761 7469  eThinSpaceNegati
0046627: 7665 5665 7279 5468 696e 5370 6163 654e  veVeryThinSpaceN
0046637: 6573 7465 6447 7265 6174 6572 4772 6561  estedGreaterGrea
0046647: 7465 724e 6573 7465 644c 6573 734c 6573  terNestedLessLes
0046657: 734e 6577 4c69 6e65 4e66 72f0 9d94 914e  sNewLineNfr....N
0046667: 6f42 7265 616b e281 a04e 6f6e 4272 6561  oBreak...NonBrea
0046677: 6b69 6e67 5370 6163 654e 6f70 66e2 8495  kingSpaceNopf...
0046687: e2ab ac4e 6f74 436f 6e67 7275 656e 74e2  ...NotCongruent.
0046697: 89a2 4e6f 7443 7570 4361 70e2 89ad 4e6f  ..NotCupCap...No
00466a7: 7444 6f75 626c 6556 6572 7469 6361 6c42  tDoubleVerticalB
00466b7: 6172 e288 a64e 6f74 456c 656d 656e 74e2  ar...NotElement.
00466c7: 8889 4e6f 7445 7175 616c e289 a04e 6f74  ..NotEqual...Not
00466d7: 4571 7561 6c54 696c 6465 e289 82cc b84e  EqualTilde.....N
00466e7: 6f74 4578 6973 7473 e288 844e 6f74 4772  otExists...NotGr
00466f7: 6561 7465 72e2 89af 4e6f 7447 7265 6174  eater...NotGreat
0046707: 6572 4571 7561 6ce2 89b1 4e6f 7447 7265  erEqual...NotGre
0046717: 6174 6572 4675 6c6c 4571 7561 6ce2 89a7  aterFullEqual...
0046727: ccb8 4e6f 7447 7265 6174 6572 4772 6561  ..NotGreaterGrea
0046737: 7465 72e2 89ab ccb8 4e6f 7447 7265 6174  ter.....NotGreat
0046747: 6572 4c65 7373 e289 b94e 6f74 4772 6561  erLess...NotGrea
0046757: 7465 7253 6c61 6e74 4571 7561 6ce2 a9be  terSlantEqual...
0046767: ccb8 4e6f 7447 7265 6174 6572 5469 6c64  ..NotGreaterTild
0046777: 65e2 89b5 4e6f 7448 756d 7044 6f77 6e48  e...NotHumpDownH
0046787: 756d 70e2 898e ccb8 4e6f 7448 756d 7045  ump.....NotHumpE
0046797: 7175 616c e289 8fcc b84e 6f74 4c65 6674  qual.....NotLeft
00467a7: 5472 6961 6e67 6c65 e28b aa4e 6f74 4c65  Triangle...NotLe
00467b7: 6674 5472 6961 6e67 6c65 4261 72e2 a78f  ftTriangleBar...
00467c7: ccb8 4e6f 744c 6566 7454 7269 616e 676c  ..NotLeftTriangl
00467d7: 6545 7175 616c e28b ac4e 6f74 4c65 7373  eEqual...NotLess
00467e7: e289 ae4e 6f74 4c65 7373 4571 7561 6ce2  ...NotLessEqual.
00467f7: 89b0 4e6f 744c 6573 7347 7265 6174 6572  ..NotLessGreater
0046807: e289 b84e 6f74 4c65 7373 4c65 7373 e289  ...NotLessLess..
0046817: aacc b84e 6f74 4c65 7373 536c 616e 7445  ...NotLessSlantE
0046827: 7175 616c e2a9 bdcc b84e 6f74 4c65 7373  qual.....NotLess
0046837: 5469 6c64 65e2 89b4 4e6f 744e 6573 7465  Tilde...NotNeste
0046847: 6447 7265 6174 6572 4772 6561 7465 72e2  dGreaterGreater.
0046857: aaa2 ccb8 4e6f 744e 6573 7465 644c 6573  ....NotNestedLes
0046867: 734c 6573 73e2 aaa1 ccb8 4e6f 7450 7265  sLess.....NotPre
0046877: 6365 6465 73e2 8a80 4e6f 7450 7265 6365  cedes...NotPrece
0046887: 6465 7345 7175 616c e2aa afcc b84e 6f74  desEqual.....Not
0046897: 5072 6563 6564 6573 536c 616e 7445 7175  PrecedesSlantEqu
00468a7: 616c e28b a04e 6f74 5265 7665 7273 6545  al...NotReverseE
00468b7: 6c65 6d65 6e74 e288 8c4e 6f74 5269 6768  lement...NotRigh
00468c7: 7454 7269 616e 676c 65e2 8bab 4e6f 7452  tTriangle...NotR
00468d7: 6967 6874 5472 6961 6e67 6c65 4261 72e2  ightTriangleBar.
00468e7: a790 ccb8 4e6f 7452 6967 6874 5472 6961  ....NotRightTria
00468f7: 6e67 6c65 4571 7561 6ce2 8bad 4e6f 7453  ngleEqual...NotS
0046907: 7175 6172 6553 7562 7365 74e2 8a8f ccb8  quareSubset.....
0046917: 4e6f 7453 7175 6172 6553 7562 7365 7445  NotSquareSubsetE
0046927: 7175 616c e28b a24e 6f74 5371 7561 7265  qual...NotSquare
0046937: 5375 7065 7273 6574 e28a 90cc b84e 6f74  Superset.....Not
0046947: 5371 7561 7265 5375 7065 7273 6574 4571  SquareSupersetEq
0046957: 7561 6ce2 8ba3 4e6f 7453 7562 7365 74e2  ual...NotSubset.
0046967: 8a82 e283 924e 6f74 5375 6273 6574 4571  .....NotSubsetEq
0046977: 7561 6ce2 8a88 4e6f 7453 7563 6365 6564  ual...NotSucceed
0046987: 73e2 8a81 4e6f 7453 7563 6365 6564 7345  s...NotSucceedsE
0046997: 7175 616c e2aa b0cc b84e 6f74 5375 6363  qual.....NotSucc
00469a7: 6565 6473 536c 616e 7445 7175 616c e28b  eedsSlantEqual..
00469b7: a14e 6f74 5375 6363 6565 6473 5469 6c64  .NotSucceedsTild
00469c7: 65e2 89bf ccb8 4e6f 7453 7570 6572 7365  e.....NotSuperse
00469d7: 74e2 8a83 e283 924e 6f74 5375 7065 7273  t......NotSupers
00469e7: 6574 4571 7561 6ce2 8a89 4e6f 7454 696c  etEqual...NotTil
00469f7: 6465 e289 814e 6f74 5469 6c64 6545 7175  de...NotTildeEqu
0046a07: 616c e289 844e 6f74 5469 6c64 6546 756c  al...NotTildeFul
0046a17: 6c45 7175 616c e289 874e 6f74 5469 6c64  lEqual...NotTild
0046a27: 6554 696c 6465 e289 894e 6f74 5665 7274  eTilde...NotVert
0046a37: 6963 616c 4261 72e2 88a4 4e73 6372 f09d  icalBar...Nscr..
0046a47: 92a9 4e74 696c 6465 c391 4e75 ce9d 4f45  ..Ntilde..Nu..OE
0046a57: 6c69 67c5 924f 6163 7574 65c3 934f 6369  lig..Oacute..Oci
0046a67: 7263 c394 4f63 79d0 9e4f 6462 6c61 63c5  rc..Ocy..Odblac.
0046a77: 904f 6672 f09d 9492 4f67 7261 7665 c392  .Ofr....Ograve..
0046a87: 4f6d 6163 72c5 8c4f 6d65 6761 cea9 4f6d  Omacr..Omega..Om
0046a97: 6963 726f 6ece 9f4f 6f70 66f0 9d95 864f  icron..Oopf....O
0046aa7: 7065 6e43 7572 6c79 446f 7562 6c65 5175  penCurlyDoubleQu
0046ab7: 6f74 65e2 809c 4f70 656e 4375 726c 7951  ote...OpenCurlyQ
0046ac7: 756f 7465 e280 984f 72e2 a994 4f73 6372  uote...Or...Oscr
0046ad7: f09d 92aa 4f73 6c61 7368 c398 4f74 696c  ....Oslash..Otil
0046ae7: 6465 c395 4f74 696d 6573 e2a8 b74f 756d  de..Otimes...Oum
0046af7: 6cc3 964f 7665 7242 6172 e280 be4f 7665  l..OverBar...Ove
0046b07: 7242 7261 6365 e28f 9e4f 7665 7242 7261  rBrace...OverBra
0046b17: 636b 6574 e28e b44f 7665 7250 6172 656e  cket...OverParen
0046b27: 7468 6573 6973 e28f 9c50 6172 7469 616c  thesis...Partial
0046b37: 44e2 8882 5063 79d0 9f50 6672 f09d 9493  D...Pcy..Pfr....
0046b47: 5068 69ce a650 69ce a050 6c75 734d 696e  Phi..Pi..PlusMin
0046b57: 7573 c2b1 506f 696e 6361 7265 706c 616e  us..Poincareplan
0046b67: 6550 6f70 66e2 8499 5072 e2aa bb50 7265  ePopf...Pr...Pre
0046b77: 6365 6465 73e2 89ba 5072 6563 6564 6573  cedes...Precedes
0046b87: 4571 7561 6ce2 aaaf 5072 6563 6564 6573  Equal...Precedes
0046b97: 536c 616e 7445 7175 616c e289 bc50 7265  SlantEqual...Pre
0046ba7: 6365 6465 7354 696c 6465 e289 be50 7269  cedesTilde...Pri
0046bb7: 6d65 e280 b350 726f 6475 6374 e288 8f50  me...Product...P
0046bc7: 726f 706f 7274 696f 6e50 726f 706f 7274  roportionProport
0046bd7: 696f 6e61 6ce2 889d 5073 6372 f09d 92ab  ional...Pscr....
0046be7: 5073 69ce a851 554f 5422 5166 72f0 9d94  Psi..QUOT"Qfr...
0046bf7: 9451 6f70 66e2 849a 5173 6372 f09d 92ac  .Qopf...Qscr....
0046c07: 5242 6172 72e2 a490 5245 47c2 ae52 6163  RBarr...REG..Rac
0046c17: 7574 65c5 9452 616e 67e2 9fab 5261 7272  ute..Rang...Rarr
0046c27: e286 a052 6172 7274 6ce2 a496 5263 6172  ...Rarrtl...Rcar
0046c37: 6f6e c598 5263 6564 696c c596 5263 79d0  on..Rcedil..Rcy.
0046c47: a052 65e2 849c 5265 7665 7273 6545 6c65  .Re...ReverseEle
0046c57: 6d65 6e74 e288 8b52 6576 6572 7365 4571  ment...ReverseEq
0046c67: 7569 6c69 6272 6975 6de2 878b 5265 7665  uilibrium...Reve
0046c77: 7273 6555 7045 7175 696c 6962 7269 756d  rseUpEquilibrium
0046c87: e2a5 af52 6672 5268 6fce a152 6967 6874  ...RfrRho..Right
0046c97: 416e 676c 6542 7261 636b 6574 e29f a952  AngleBracket...R
0046ca7: 6967 6874 4172 726f 77e2 8692 5269 6768  ightArrow...Righ
0046cb7: 7441 7272 6f77 4261 72e2 87a5 5269 6768  tArrowBar...Righ
0046cc7: 7441 7272 6f77 4c65 6674 4172 726f 77e2  tArrowLeftArrow.
0046cd7: 8784 5269 6768 7443 6569 6c69 6e67 e28c  ..RightCeiling..
0046ce7: 8952 6967 6874 446f 7562 6c65 4272 6163  .RightDoubleBrac
0046cf7: 6b65 74e2 9fa7 5269 6768 7444 6f77 6e54  ket...RightDownT
0046d07: 6565 5665 6374 6f72 e2a5 9d52 6967 6874  eeVector...Right
0046d17: 446f 776e 5665 6374 6f72 e287 8252 6967  DownVector...Rig
0046d27: 6874 446f 776e 5665 6374 6f72 4261 72e2  htDownVectorBar.
0046d37: a595 5269 6768 7446 6c6f 6f72 e28c 8b52  ..RightFloor...R
```

These I assume come from junkdog's tachyonfx, which I really dig, and use for
the intro animation. I could use my own code to create an intro animation, but I
wonder if there is some kind of way to only add format strings that I actually
use to the data section, since it would shrink it by a ton. I believe I am on a
good track here, but not sure if it's possible.

I went back to claude and found the following:
```
wasm-opt -Oz \
  --enable-bulk-memory \
  --enable-nontrapping-float-to-int \
  --enable-mutable-globals \
  --enable-sign-ext \
  --enable-simd \
  --enable-reference-types \
  --strip-debug \
  --strip-producers \
  --strip-target-features \
  --remove-unused-names \
  --remove-unused-module-elements \
  --merge-blocks \
  --coalesce-locals \
  --simplify-locals \
  --vacuum \
  --dae-optimizing \
  ../dist/rbn-dev-4be7aa163e2579ce_bg.wasm -o ../dist/optimized.wasm
  ```

  It shaved another 18K off of the bin but the data section is still, le bloated
  I guess, I will have to strip the tachyonfx lib completely.

  After stripping tachyonfx and letting claude make some intro animation I'm 
  down to 352143 bytes or 344K. Getting closer once again :D.

 > Note to self:
 > TODO 
 > Add above to build script
 > Experiment with wasm-snip
 > Remove tachyonfx for manual rendering of fx.
 > Do https://github.com/johnthagen/min-sized-rust?tab=readme-ov-file#remove-fmtdebug
