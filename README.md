# *Wiregen* - an efficient binary protocol generator

**DISCLAIMER:** This is an ongoing *work in progress*, specifications may change, there may
be bugs and incomplete functionality.

## Five-minute bootstrap

 Download the sources, build them by invoking `cargo build` in the repository root. Then
you may use either `cargo run --` appending command line parameters, or grab the built
executable and use it directly.

 Typical code generation invocation looks like follows (C++ is currently the only
supported language target).

``` shell
wiregen write schema.rs --output protocol.cpp --lang cpp
```

Schema examples are provided in the `examples/` folder.

## Table of contents

 * [High level overview](#high-level-overview)
 * [Design](#design)
   * [Rust as a schema language](#rust-as-a-schema-language)
   * [Rust as an implementation language](#rust-as-an-implementation-language)
 * [Improvement scope](#improvement-scope)

## High-level overview

*wiregen* is a standalone native console application, whose main purpose is to generate
source files in a variety of programming languages, implementing an efficient binary
protocol defined by a data model.

This is a very similar idea to to [Google Protocol Buffers](https://github.com/google/protobuf),
which is a self-contained app called `protoc` that generates
C++/Java/C#/etc. implementations of a schema-defined versioning-resistant protocol.

However, Protocol Buffers is more about interop in a heterogenous environment where
individual components are updated asynchronously, thus it pays significant attention to
field order and ability to unpack the binary blob in case of version difference on
receiving/sending ends. All of this makes it perfect for its stated goal, and for tasks
like persisting some data for a long time. This comes at a cost of efficiency on the wire,
albeit it's still much better than the usual text-based suspects like JSON/XML/etc. The
same can be said about typical `protoc` alternatives,
like [FlatBuffers](https://google.github.io/flatbuffers/), [Cap'n Proto](capnproto.org), etc.

*wiregen*, on the other hand, strives to provide a wire format and an ORM for a
typical realtime video game networking protocol, which is very different from Protocol
Buffers intended domain:

  * Given that we not target persistence (a task which is solved nicely enough by existing
    solutions) versioning is not important, as binary blobs sent over the wire live for
    mere milliseconds while in-flight between fully version-synchronized client & server.

  * Efficient coding and data compression matters a lot more, as it allows to stuff more updates
    into a single packet without exceeding an MTU. It's generally agreed that
    fragmentation is a no-go for most games, and increasing the update rate has its costs
    in L2/L3 packet header overhead.

  * Game protocols typically operate over UDP, where packet loss and reordering may
    happen. This is dictated by strict latency requirements, and mandates special handling
    on every level of the design. As an illustrative example, existing data compression
    middleware is usually of no help, and custom algorithms like delta bit packing are
    required.

  * There are additionally problems intrinsic to the domain, like interpolation and data
    hiding, which are not tackled by existing solutions at all.

Historically, such network protocols are custom written for each game and are heavily tied
to a specific middleware or even title. *wiregen* aims to provide a solution that is good
enough out-of-the-box, fills the gaps which are often left unattended in homebrew
solutions (like, most notably, information hiding), and possibly enable novel applications
like heterogeneous client/servers (for example, Unity client connecting directly to C++
server).

 *wiregen* **is not** designed to be a full networking library, it assumes basic transport
functionality from lower layer. This requirements are trivial, however - an ability to
send/receive a binary blob in a UDP datagram will be enough.

## Design

 This is a project which uses [Rust](https://www.rust-lang.org/en-US/) programming
language in two ways - as an implementation language and as a schema definition
format. The latter means, to make a Protocol Buffer analogy, that *wiregen* counterpart to
a `.proto` file is a `.rs` source file of limited syntax. This rather atypical design
decisions are justified below.

### Rust as a schema language

* Rust has a rapidly evolving ecosystem of IDEs, text editor plugins, linters, etc., which
  means that programmers authoring *wiregen* schemas will not be left in the wild with a
  plain text editing functionality without even basic syntax coloring, which is what will
  happen if we decide to go custom text format. Editor support, at least in form of
  _vi_ or _Emacs_ plugins, is certainly doable, but this is a problem that can take
  formidable effort in itself. It's much better to leverage something existing.

* Another alternative would be to use Protocol Buffers schema format, which also has wide
  IDE support and a lot of engineers are familiar with it. This would make *wiregen* a
  ProtoBuf backend of sorts - a custom implementation centered on wire efficiency and game
  networking requirements. While this possibility was considered early in the design, it
  quickly became obvious that `.proto` format cannot express a lot of required features
  without resorting to very kludgy hacks.

* Assuming Rust may become a first class player of game development scene in the coming
  years (things are certainly looking this way) and its support of procedural macros
  (arbitrary compile-time code transformations), *wiregen* targeting Rust should not even
  be a standalone code generator - it will be just
  a [macro crate](https://doc.rust-lang.org/book/first-edition/procedural-macros.html).

* Using annotated general-purpose language source as a schema definition is not a novel
  idea - [Qt Moc](https://doc.qt.io/archives/3.3/moc.html) has been around for ages. Among
  alternative design choices of annotating C++, C#, Java sources, Rust has the following
  benefits:

  * Given Rust is a target language for *wiregen*, it's sort of 'least common denominator'
    one - translating arbitrary C#/C++ hierarchies into Rust and other subtyping-emulation
    tasks are not trivial and are very unnatural in idiomatic Rust, as it is not an OOP
    language (at least not in a conventional way)

  * Rust supports sum types, optionals, many types of collections out of the box, while
    making performance-influencing distinctions explicit. This features are generally
    considered important, but C++ has got some of them in C++17 only (with much worse
    ergonomics), and C#/Java still luck idiomatic sum types, for example.

  * Rust has well-defined, concise syntax, a rich type system and an attribute
    mechanism - all of it allows expressing arbitrary intents (especially compared to
    `.proto` format), parsing source code easily (especially compared to C++), and reduces
    visual bloat.

  * Rust module system lends itself particularly well to splitting large protocol
    definitions over multiple files.

``` rust
// An example implementing heterogenous world position format

#[qlinspace(low="-1.2e40",high="1e4",step="0.1")]
type XZ = f32;

#[qlinspace(low="-100", high="100", step="0.1")]
type Y = f32;

struct WorldPosition {
    x: XZ,
    y: Y,
    z: XZ
}
```

### Rust as an implementation language

* There is a Rust parser in Rust called [syn](https://github.com/dtolnay/syn), generic
  parsing combinator library [nom](https://github.com/Geal/nom), and a plethora of quoting
  and pretty printing libraries. This makes manipulating Rust source/AST in Rust very
  convenient.

* The built-in support for sum types, optionals/errors, idiomatic combinators and pattern
  matching allow to express a lot of AST transformations very
  cleanly and bug free. Quoting [Panopticon](https://panopticon.re/) author:

    > I toyed with the idea of rewriting it in Rust since 1.0 became stable. The whole
    > port took around 3 months. I got the size down from 10.000 to 8.000 loc. Looking
    > back it was the right decision. Programming in Rust is not only more fun, it's
    > definitely easier too. Panopticon used alot sum types that were implemented using
    > boost::variant. Like everything with Boost it kind of worked but was incredible ugly
    > and complex. Replacing them with enums was probably the biggest reason I switched to
    > Rust. Also I found iterator invalidation bugs simply by translating C++ to Rust,
    > thanks Borrow Checker!

* Compared to dynamic languages, Rust static typing catches *lots* of bugs early.

* Rust templating engine [askama](https://github.com/djc/askama) is strongly-typed, very
  performant and has feature parity with typical alternatives like Jinja.

* Rust is a native language. Apart from C++-like performance, which is not important in
  this context but is always something nice to have, Rust delivers a single binary with no
  dependencies.

* Rust [package-management infrastructure](https://crates.io/) is top-notch and on par
  with offerings for dynamic languages.

## Improvement scope

 Current version strives to be an MVP and sports a very limited feature set:

  * C++ is currently the only supported language
  * The data compression is achieved by mere fixed-length bit packing and delta coding via
    difference flags (yet this still manages to beat Mercury by a wide margin)
  * It's mere serialization protocol for now, with no attempt to solve packet loss,
    interpolation/dead reckoning and information hiding issues
  * No ORM-like data model
  * No callbacks on changes
  * Data model is limited to:
      * Basic integers and floats
      * C-style enums
      * POD structs
      * Generic blobs
      * Bounded arrays (not vectors)
  * Control over wire representation is limited to linear/wrapping quantization for floats
    and bounding for integers.

  There has been a significant amount of experiments done already on various design
  aspects, which are yet to find their way into implementation. The following gives a
  rough idea what's in store for future versions, in the order of most probable delivery:

  1. Sum types and "narrowing" - this is required to implement proper information
     hiding. Sum types (Rust-style enums) are convenient on their own.

  2. Extension hooks for the data model - an ability to call back on any change.

  3. More versatility from built-in types - vector/map collections
     foremost, possibly common linear algebra like
     matrices/quaternions that is extremely convenient for gamedev
     applications.

  4. C# support - this is the first in line among target languages

  5. State-of-the-art data compression - there are experiments on
     using [rANS](https://github.com/rygorous/ryg_rans) to great success for general
     purpose tasks, and a prototype [FSE](https://github.com/Cyan4973/FiniteStateEntropy)
     implementation that will put *wiregen* forward in terms of wire efficiency.

  6. Proper data-model - ability for end user to abstract away from protocol and
     transport, and just use some object hierarchy via reading/setting properties and
     invoking methods. This is the most crucial step for the adoption of *wiregen*.

  7. Interpolation/dead reckoning/channel abstractions - this is very hard to generalize,
     but on the same hand a huge source of boilerplate in each and every networked game.
