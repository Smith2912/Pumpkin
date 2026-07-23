# Loader compatibility adapters

## Status

This document is a design record, not a claim that Fabric, Quilt, Forge, or
NeoForge mods currently run on Pumpkin. Implementation starts only after the
Bukkit/Spigot/Paper release gates in `PLUGIN_API_COMPATIBILITY.md` pass on the
deployed test server.

## Decision

Pumpkin will use one loader-neutral bridge kernel and separate compatibility
front ends for each Java API family:

- Bukkit, Spigot, and Paper: PatchBukkit;
- Fabric and Quilt: one adapter family with small loader-specific entrypoint
  and metadata shims;
- Forge and NeoForge: one adapter family with small loader-specific lifecycle,
  metadata, and version shims.

The families must not load into one shared Java class path. Each family owns
its loader, mappings, dependencies, and version selection. A server instance
selects one Java compatibility family at startup.

This avoids treating similarly named concepts as interchangeable. For example,
Fabric events are not Forge event-bus events, Fabric components are not Forge
capabilities or NeoForge data attachments, and Bukkit services are not mod
loader registries.

## Existing Pumpkin boundaries to reuse

The adapter work extends existing boundaries rather than creating a parallel
server API:

| Concern | Existing Pumpkin boundary | Adapter responsibility |
| --- | --- | --- |
| Discovery and lifetime | `plugin::PluginManager`, `PluginLoader`, `Plugin` | Read family metadata, resolve dependencies, start and stop the Java runtime |
| Server access | `plugin::api::Context` | Expose stable, capability-scoped bridge services instead of Rust internals |
| Events | `plugin::api::events::Payload`, `EventHandler`, `EventPriority` | Translate event data, cancellation, ordering, and thread rules |
| Commands | `command::node::dispatcher::CommandDispatcher` and fallback dispatcher | Translate argument trees, permissions, suggestions, execution, and results without replacing native commands |
| Permissions and services | `Context` permission and service methods | Map the family’s service or permission model without inventing global defaults |
| Scheduling | Pumpkin server/tick scheduling exposed through a bridge service | Preserve main-thread, asynchronous, repeating-task, and cancellation semantics |
| Players and worlds | `Server`, `Player`, world/entity/inventory APIs | Use stable handles and value messages; never expose Rust object addresses |
| Networking | Java and Bedrock network boundaries plus packet events | Register channels/codecs and validate direction, phase, size, and player ownership |
| Native and Wasm plugins | Existing native and Wasm loaders | Remain independent and continue to work when no Java adapter is selected |

`Context` currently exposes some concrete server objects. New Java bridge
contracts should be narrower service interfaces so Java compatibility does not
freeze Pumpkin’s internal Rust layout.

## Bridge kernel

The bridge kernel is the only layer allowed to call Pumpkin internals on behalf
of Java compatibility code. Its contracts are versioned independently of the
Java APIs and grouped by behavior:

1. Runtime: handshake, API version, capabilities, health, shutdown, and
   structured errors.
2. Lifecycle: discovery result, dependency graph, load/enable/disable phases,
   and failure isolation.
3. Scheduler: server-thread execution, asynchronous work, delayed/repeating
   tasks, ownership, cancellation, and shutdown draining.
4. Commands: namespaced registration, collision policy, aliases, permissions,
   suggestions, sender identity, result, and visible error delivery.
5. Events: event identity, priority, cancellation, mutable fields, synchronous
   versus asynchronous delivery, and exception isolation.
6. Registries: stable resource keys, snapshots, tags, dynamic reload epochs,
   and explicit unsupported operations.
7. Game state: players, worlds, chunks, entities, inventories, items, recipes,
   scoreboards, bosses, configuration, and persistent data.
8. Networking: channel negotiation, packet phase/direction, payload limits,
   disconnect behavior, and versioned codecs.

Every bridge call returns a result with a stable error code and human-readable
context. Unsupported behavior must return `UNSUPPORTED`; it must not silently
succeed, return fabricated state, or leave an asynchronous failure only in a
background log.

Long-lived objects cross the boundary as generation-checked handles. Handles
become invalid when a player disconnects, an entity is removed, a world is
unloaded, or an adapter restarts. The Java side must not retain native
references beyond their documented lifetime.

## Adapter runtime model

The first implementation should reuse PatchBukkit’s proven native-plugin plus
Java-runtime pattern, but move shared transport and process supervision into
the bridge kernel. This is an architectural reuse, not permission to share
Bukkit classes with other families.

Each selected family receives:

- a private class path and dependency resolver;
- a loader-specific metadata parser and dependency graph;
- one version/mappings module per supported Minecraft line;
- an explicit startup handshake listing implemented bridge capabilities;
- isolated logs, configuration, persistent data, and crash reporting;
- a deterministic stop sequence that rejects new work, cancels owned tasks,
  disables extensions in reverse dependency order, and releases handles.

Hot reload is not a baseline requirement for Java mods. The existing Pumpkin
plugin watcher must not attempt to reload Java mods whose loader contract does
not support safe unloading.

## Fabric and Quilt family

The shared layer covers public concepts that both loaders intentionally expose:

- metadata and entrypoints;
- lifecycle callbacks;
- event callbacks;
- commands and argument registration;
- registries and tags;
- networking channels and payload codecs;
- resource/data reload listeners;
- configuration and persistent component-like data where an installed API
  defines it.

Thin Fabric and Quilt shims own loader-specific metadata, discovery,
entrypoints, dependency/version rules, and API-package differences.

Mixins, access wideners, intermediary/Yarn/Mojang mapping references, direct
Minecraft implementation classes, and loader internals are a separate
compatibility track. A mod is not counted as public-API compatible merely
because its initializer ran before a Mixin failed.

## Forge and NeoForge family

The shared layer covers:

- mod metadata and dependency ordering;
- mod event bus and gameplay event bus;
- deferred and dynamic registries;
- commands;
- configuration lifecycle;
- networking channels, payload registration, and version negotiation;
- capabilities and NeoForge data attachments through distinct adapter
  interfaces;
- server lifecycle and inter-mod communication where its contract can be
  preserved.

Thin Forge and NeoForge shims own loader-specific metadata, lifecycle phase
names, event types, registry mechanics, networking versions, and API-package
differences.

Capabilities, attachments, and Fabric-style components must remain distinct
front-end abstractions even if all three persist data through the same Pumpkin
storage service.

Coremods, access transformers, Mixin use, bytecode transformers, mapping-bound
Minecraft classes, and reflective loader internals are a separate
compatibility track.

## Versioning and source layout

The planned source boundaries are:

```text
pumpkin/src/plugin/bridge/             stable Pumpkin-facing services
compat/java/common/                    transport, handles, errors, supervision
compat/java/bukkit-paper/              PatchBukkit build and adapter
compat/java/fabric-quilt/common/       shared public-API implementation
compat/java/fabric-quilt/fabric-<mc>/  Fabric/version shim
compat/java/fabric-quilt/quilt-<mc>/   Quilt/version shim
compat/java/forge-neoforge/common/     shared public-API implementation
compat/java/forge-neoforge/forge-<mc>/ Forge/version shim
compat/java/forge-neoforge/neoforge-<mc>/ NeoForge/version shim
compat/conformance/                    contracts and real-extension fixtures
```

Those paths are proposed boundaries, not a requirement to move PatchBukkit
before its current release gates pass.

Bridge protocol changes follow additive versioning:

- add fields and calls without changing existing meaning;
- negotiate capabilities during startup;
- retain at least the current and previous bridge protocol while a migration is
  active;
- reject an incompatible runtime before any extension entrypoint executes;
- isolate Minecraft-version and mapping changes inside the version shim.

## Completion gates

An adapter family is usable only when all of these gates pass:

1. A clean, documented build produces the exact runtime artifact.
2. Startup reports selected family, Minecraft version, adapter version, bridge
   version, capabilities, and discovered extensions.
3. Dependency ordering and missing/incompatible dependency failures are
   deterministic and readable.
4. A conformance extension proves lifecycle, scheduler ownership, command
   routing, event mutation/cancellation, registry lookup, configuration,
   persistence, player/world state, and networking behavior.
5. At least one representative real extension for each claimed behavior works
   on a deployed server.
6. A human performs the documented in-game action and records the visible
   result, relevant log line, deployed commit, and timestamp.
7. Stop/restart leaves no owned tasks, stale handles, partial writes, or
   orphaned Java runtime.
8. Unsupported features are listed with stable errors and are not represented
   as working.

Compilation, JAR discovery, a successful entrypoint, or a passing mock alone
does not satisfy a behavior gate.

## First implementation slices

After the Bukkit/Paper gate passes:

1. Extract the PatchBukkit transport, handle, error, and supervision contracts
   into the bridge kernel without changing Bukkit behavior.
2. Add a bridge protocol handshake and capability report, then replay the
   Bukkit conformance suite to prove no regression.
3. Build a Fabric/Quilt discovery-and-lifecycle vertical slice with one
   command, one cancellable event, one scheduled task, and one persistent value.
4. Build the equivalent Forge/NeoForge vertical slice.
5. Use the two slices to settle shared bridge contracts before broad API
   implementation.

Each slice is one reviewable batch with one meaningful validation pass and one
deployment. Broad API work follows failures from the conformance and real-mod
matrices, not raw counts of stubbed methods.
