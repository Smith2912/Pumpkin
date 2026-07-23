# Plugin API compatibility

Pumpkin's plugin compatibility target is behavioral compatibility with the
current Bukkit, Spigot, and Paper APIs. A plugin is not considered compatible
merely because its JAR loads.

## Release gates

A compatibility milestone must satisfy all of these gates:

1. Plugins are discovered in the standard plugin directory.
2. Hard dependencies are loaded and enabled first.
3. Declared dependency classes are shared without duplicate class identities.
4. `onLoad`, `onEnable`, and `onDisable` run in Bukkit order.
5. Registered commands, services, permissions, schedulers, and events work.
6. Player, entity, inventory, and world operations change Pumpkin state.
7. Startup and shutdown contain no compatibility exceptions.
8. The conformance plugin and the real-plugin test matrix pass.

## Implementation milestones

- Runtime: server identity, plugin metadata, dependency graph, class loading,
  lifecycle, configuration, libraries, and safe shutdown.
- Server services: commands, services, permissions, scheduler, messaging,
  configuration, and plugin lifecycle events.
- Player lifecycle: pre-login, login, join, quit, command, chat, world-change,
  game-mode, kick, teleport, death, and respawn events.
- Gameplay API: players, entities, worlds, blocks, inventories, items,
  recipes, scoreboards, bosses, advancements, and persistent data.
- Paper API: Adventure audiences, asynchronous operations, command syncing,
  modern events, registries, profiles, and region-safe scheduling.

## Build and deployment cadence

Compatibility work ships in behavior-sized batches rather than one missing
method per deployment. A batch must:

1. Complete one user-visible workflow, including the adjacent reads, writes,
   events, and state synchronization that a real plugin reaches.
2. Keep unrelated API areas in separate commits so a maintainer can review,
   revert, and verify the behavior independently.
3. Pass the clean pinned-patch, Java/protobuf, Rust, and exact-source build
   gates once for the complete batch.
4. Produce one Railway deployment candidate and one focused live verification
   record for that exact commit.
5. Use live failures to choose the next batch instead of immediately shipping
   another isolated stub replacement.

Boot failures, data-loss risks, and similarly urgent regressions may ship as
smaller hotfixes. Batch size never relaxes the release gates below.

## Real-plugin matrix

- LuckPerms
- EssentialsX core and official modules
- Vault
- PlaceholderAPI
- WorldEdit
- ViaVersion

Plugins that directly use CraftBukkit or Minecraft internals require an
additional internal-API shim. Their compatibility is tracked separately from
public Bukkit, Spigot, and Paper API compatibility.

## Inventory milestone gate

The player-inventory milestone is complete only when one deployed commit
passes all of these checks:

1. Bukkit can read and write all 41 player slots: hotbar, storage, armor, and
   off-hand.
2. `Registry.ITEM`, modern Bukkit `ItemStack` constructors, and empty stacks
   produce usable Pumpkin-backed item objects with native stack limits.
3. Bukkit can read and write the selected hotbar slot, cursor item, and all 27
   ender-chest slots.
4. `addItem`, `removeItem`, `clear`, contents/storage accessors, equipment
   accessors, and iterators preserve Bukkit slot semantics.
5. The owning client sees mutations immediately, and held/armor/off-hand
   changes are visible to other players.
6. Untouched Pumpkin item components survive Bukkit reads, amount changes, and
   writes without loss.
7. Focused native round-trip tests, clean Java/protobuf compilation, Rust
   compilation, clean pinned-patch application, and a live EssentialsX
   give/clear/equipment/ender-chest workflow all pass.

Opaque native component preservation does not by itself implement arbitrary
Bukkit `ItemMeta` mutation.

## Item-metadata milestone gate

The Essentials-critical metadata subset is complete only when one deployed
commit passes all of these checks:

1. `Server.getItemFactory()` and `ItemStack#getItemMeta()` return
   server-owned, cloneable metadata objects.
2. Display name, lore, enchantments, durability damage, repair cost, and
   unbreakable state round-trip from Pumpkin to Bukkit and back.
3. `Registry.ENCHANTMENT` is populated from Pumpkin and Bukkit's built-in
   enchantment constants use those registry instances without recursive static
   initialization.
4. Applying supported metadata preserves every unrelated native item
   component.
5. Unsupported metadata mutations fail explicitly rather than appearing to
   save data that the bridge discards.
6. Focused native component and bridge round-trip tests, clean
   Java/protobuf compilation, Rust compilation, clean pinned-patch
   application, and live EssentialsX item-name/lore/enchant/repair workflows
   all pass.

This milestone does not include specialized metadata such as books, maps,
potions, skull profiles, banners, fireworks, armor trims, or modern Paper
data-component mutation. Item persistent-data writes and rich text formatting
also remain explicit follow-up contracts until they have native storage,
round-trip tests, and live plugin evidence.

## Block and interaction event milestone gate

The EssentialsAntiBuild/EssentialsProtect block-event milestone is complete
only when one deployed commit passes all of these checks:

1. `World.getBlockAt()` exposes the live Pumpkin world, coordinates, material,
   and basic block predicates without creating a second world model.
2. Native block and interaction events carry the real world UUID, block
   position, material, player, and action into Bukkit.
3. Bukkit cancellation prevents the corresponding Pumpkin action, and
   mutable values such as block-break experience and item drops propagate
   back to Pumpkin.
4. Non-player world changes are not misreported as player block-break events.
5. Unsupported block writes and unbridged event fields fail explicitly or
   remain documented; they must not appear to work while discarding changes.
6. The clean Java/protobuf and Rust builds, clean pinned-patch application,
   conformance block-read check, and live protected build/break/interact
   workflow all pass.

The first slice covers live block reads, `BlockBreakEvent`,
`BlockPlaceEvent`, and `PlayerInteractEvent`. Placement carries the real
placed position, placed material, player, cancellation, `canBuild`, and the
live replaced block state. Pumpkin does not yet expose the placed-against
position, face, or hand, so those fields use documented placeholders and are
not counted as complete. Piston, explosion,
entity-damage, hanging, dispense, craft, pickup, and drop events remain
separate native contracts and are not represented as complete by listener
registration alone.

## Block protection and environment batch gate

The next native block batch extends the protected-build workflow without
claiming unrelated environmental compatibility:

1. `BlockCanBuildEvent` carries the real target position, proposed material,
   player, and initial native build decision. A Bukkit listener's updated
   `buildable` value determines whether Pumpkin continues placement.
2. `BlockPlaceEvent#setBuild(false)` prevents placement even when the event is
   not separately cancelled.
3. Natural fire destruction fires `BlockBurnEvent` with the real world,
   burning position, igniting fire position, and both captured materials.
   Cancelling the Bukkit event prevents the block mutation and TNT priming.
4. Natural scheduled fire spread fires `BlockIgniteEvent` with `SPREAD`, the
   real target position, and the source fire position. Cancellation prevents
   Pumpkin from placing the new fire block.
5. Pressure-plate power changes fire `BlockRedstoneEvent`; a Bukkit listener's
   bounded `newCurrent` value is applied by Pumpkin.
6. The conformance plugin registers all four event types, and live logs must
   contain their native registration records without matching unsupported
   event warnings.
7. Human verification must show a denied placement remaining unchanged,
   cancelled burn/ignite events preserving their target blocks, and a modified
   pressure plate signal producing the expected visible redstone state.

This batch does not yet cover player-created ignition, lava ignition, fluid
flow, pistons, explosions, arbitrary redstone components, or fire paths other
than the natural scheduled fire-spread path. Those remain explicit contracts
rather than being inferred from `BlockBurnEvent` and `BlockIgniteEvent`
support.

## Fluid-flow event batch gate

The native fluid contract is complete for ordinary water and lava spreading
only when:

1. Every downward and horizontal target selected by Pumpkin fires
   `BlockFromToEvent` with the real world and exact source/target positions.
2. Bukkit cancellation prevents that target mutation before block replacement,
   lava/water conversion, or fluid tick scheduling.
3. Cancelling one horizontal target does not suppress independent flow targets
   selected during the same tick.
4. The conformance plugin registers the event, live logs contain its native
   registration without an unsupported warning, and a human can visibly keep
   water and lava out of protected target blocks.

This contract does not cover bucket placement, dispensers, sponge absorption,
waterlogging, cauldron changes, entity movement in fluids, or plugins changing
the destination block. Those paths remain separate, explicit milestones.

## Player state and teleport batch gate

The next player-state batch is complete only when one deployed commit passes
all of these checks:

1. A same-world Pumpkin teleport fires exactly one `PlayerTeleportEvent` with
   the live player, world, origin, destination, yaw, pitch, and the narrowest
   native cause Pumpkin knows. Cancellation prevents all position, rotation,
   camera, and client-packet changes.
2. Bukkit listeners may update the same-world destination coordinates, yaw,
   and pitch, and Pumpkin applies those resolved values. A listener may not
   replace the destination world on this native same-world path.
3. Bukkit-originated player teleports fire the Bukkit event before entering
   Pumpkin and then use a post-event native path. That path must not recursively
   re-enter the JVM worker. Both same-world and cross-world plugin destinations
   preserve the Bukkit listener's resolved location.
4. Initial spawn, respawn synchronization, and the post-world-change client
   handshake use the non-event position synchronizer and do not emit duplicate
   or spurious teleport events.
5. Java and Bedrock flight, sneak, sprint, and held-hotbar-slot transitions
   fire their matching Bukkit events before mutation. Cancellation preserves
   the previous state and sends the client correction required for flight or
   held-slot changes.
6. `PlayerExpChangeEvent` carries the real XP delta and a Bukkit listener's
   updated amount is used by Pumpkin's level/point calculation.
7. The conformance plugin registers all six new Bukkit event types. Live logs
   contain native registrations with no matching unsupported-event warnings,
   and no teleport action stalls the JVM worker.
8. Human verification covers an Essentials teleport, a cancelled teleport, a
   game-mode change out of spectator, a cancelled flight/held-slot transition,
   and an XP award whose amount is visibly modified.

Pumpkin-originated cross-world movement still uses the separate native
`PlayerChangeWorldEvent`/`PlayerChangedWorldEvent` lifecycle and is not yet
reported as a Bukkit `PlayerTeleportEvent`. Native callers that cannot identify
a more specific reason use Bukkit's `UNKNOWN` cause. Teleport flags, relative
teleports, vehicles/passengers, `PlayerMoveEvent`, portals, beds, and death
transitions remain separate contracts.

## Online player controls batch gate

The Essentials-style online-player controls batch is complete only when one
deployed commit passes all of these checks:

1. Bukkit `getAllowFlight`, `setAllowFlight`, `isFlying`, `setFlying`,
   `getFlySpeed`, `setFlySpeed`, `getWalkSpeed`, and `setWalkSpeed` use the
   live Pumpkin ability state. Disabling flight also stops active flight, and
   every accepted change immediately sends the client ability update.
2. Bukkit speeds reject non-finite values and values outside `[-1, 1]` before
   crossing the bridge. Pumpkin independently rejects invalid native speeds
   and rejects `flying=true` when flight is not allowed.
3. Bukkit `isSprinting`, `setSprinting`, `isSneaking`, and `setSneaking` use
   one player-session bridge. A change updates both Pumpkin's entity state and
   the metadata visible to clients.
4. Bukkit `getPing`, `getLocale`, and `locale` read the connected player's live
   keepalive latency and client locale without maintaining a duplicate Java
   cache.
5. `Player.performCommand` executes known Bukkit plugin commands directly in
   the Java command map. Known native Pumpkin commands are accepted through a
   deferred native dispatch so a plugin command cannot recursively deadlock the
   JVM worker. Unknown commands return `false`.
6. Bukkit `kickPlayer` and Paper `kick(Component, Cause)` fire a cancellable
   `PlayerKickEvent`, preserve a listener-modified reason, serialize that
   Adventure component losslessly, and disconnect through Pumpkin's normal
   edition-aware path.
7. The pinned PatchBukkit patch applies cleanly and both Java/protobuf and Rust
   bridge builds pass from that clean replay.
8. Human verification changes a connected player from spectator to survival,
   runs Essentials `/fly` twice, changes fly and walk speed, executes one
   plugin command and one native command through `Player.performCommand`, and
   confirms a cancelled kick leaves the player connected before a second kick
   disconnects with the expected reason.

`Player.performCommand` returns after a native command has been validated and
queued; native execution completes asynchronously. Bukkit plugin commands
remain synchronous inside the Java command map. The kick event's reason and
cancellation are bridged in this batch; custom kick leave-message propagation
remains a separate quit-lifecycle milestone.

## Server settings snapshot batch gate

The server-settings batch used by Essentials and administration plugins is
complete only when one deployed commit passes all of these checks:

1. One native configuration snapshot supplies Bukkit's player limit, Java
   port and bind address, view and simulation distances, MOTD, online mode,
   global flight policy, hardcore state, default game mode, forced-game-mode
   flag, End/Nether flags, IP logging policy, world type, structure generation,
   maximum world size, and spawn-protection radius.
2. Values come from Pumpkin's active configuration or an explicitly documented
   Pumpkin capability. The bridge must not keep a second Java settings cache.
   Pumpkin has no global anti-flight kick policy, so `getAllowFlight` is true;
   it has no spawn-protection radius, so `getSpawnRadius` is zero.
3. Unsigned native player limits cannot wrap into negative Bukkit values.
   Ports and distances preserve their configured values.
4. Bukkit `getDefaultGameMode` and `setDefaultGameMode` round-trip through
   Pumpkin's live default-game-mode lock. Invalid names are rejected by the
   native bridge and do not change the current value.
5. The conformance suite reads every exposed setting and temporarily changes
   the default game mode, verifies the native update, and restores the original
   value in a `finally` block.
6. The pinned patch applies cleanly and the clean replay passes the full Java,
   protobuf, test-plugin, and Rust bridge builds.
7. Human verification runs `/pbtest server`, compares the reported port,
   player limit, distances, and MOTD with the reviewed Pumpkin configuration,
   and confirms a newly joining player receives the selected default game
   mode.

Runtime changes to maximum players, MOTD, spawn radius, and the immutable base
configuration are not claimed by this gate. Their Bukkit setters remain
explicitly unsupported until Pumpkin owns mutable runtime state for the login
limit, status response, and world policy. `setDefaultGameMode` changes the live
value for this process and resets from configuration after restart.

## Player bootstrap and command-routing batch gate

The player-bootstrap batch is complete only when one deployed commit passes
all of these checks:

1. Paper's menu registry supplies every key referenced by `MenuType` without
   recursively initializing `MenuType` while `Registry.MENU` is being built.
   `InventoryType.PLAYER` and `InventoryType.ENDER_CHEST` must initialize before
   PatchBukkit constructs the first online player.
2. A player joining the server creates and registers one
   `PatchBukkitPlayer`; subsequent events and commands reuse that exact object.
   Logs must not contain `Cannot create instance of
   org.patchbukkit.entity.PatchBukkitPlayer`, `Could not initialize class
   org.bukkit.event.inventory.InventoryType`, or `Player not found for UUID`.
3. A Bukkit plugin command does not overwrite an already registered native
   Pumpkin command. The native label remains authoritative, while the plugin
   receives its lowercase `plugin:command` fallback and unclaimed aliases.
   For Essentials this means `/gamemode survival` remains Pumpkin's command
   and `/essentials:gamemode survival` remains explicitly available.
4. Java command execution reports its handled/error result back to Pumpkin.
   Missing commands and Java exceptions produce a visible player message
   instead of being logged after the native dispatcher has already reported
   success.
5. Paper Adventure `Component` messages sent to a player serialize through
   the same Pumpkin message bridge as legacy Bukkit strings. Essentials
   success and permission messages must be visible in game.
6. Player display name, player-list name/header/footer/order, and action-bar
   messages round-trip through Pumpkin-owned state. The Java bridge must not
   keep a second presentation cache.
7. The focused menu-bootstrap probe resolves all 25 Paper menu keys,
   `MenuType.GENERIC_9X3`, and `InventoryType.PLAYER`; the command-route test
   proves native collision preservation and plugin fallback aliases.
8. The pinned patch applies cleanly and the clean replay passes the Java,
   protobuf, test-plugin, and Rust bridge builds.
9. Human verification joins with one test player, confirms the three startup
   exceptions above are absent, runs `/gamemode survival`, verifies the HUD and
   movement change out of spectator mode, and runs
   `/essentials:gamemode spectator` followed by
   `/essentials:gamemode survival` with visible Essentials feedback.

Menu identifiers are implemented in this batch so the Paper API can bootstrap
and inventories can report their type. Paper menu-view builders remain
explicitly unsupported until Pumpkin has a native container/view construction
bridge; this gate does not claim them.

## Human verification runbook

Every compatibility change must leave evidence a maintainer can reproduce and
understand. Test count alone is not evidence.

### Reproducible build

1. Confirm `PATCHBUKKIT_COMMIT` in the root `Dockerfile` is the reviewed
   PatchBukkit revision.
2. With clean Pumpkin and PatchBukkit checkouts, run:

   ```text
   git -C <patchbukkit-checkout> apply --check --unidiff-zero <pumpkin-checkout>/docker/patchbukkit-26.2.patch
   ```

3. Apply the patch and run PatchBukkit's full Java build. This verifies the
   Java API surface, generated protobuf classes, and conformance plugin.
4. Build the root Docker image. The image build compiles Pumpkin and
   PatchBukkit's native bridge in release mode against the same local Pumpkin
   crates; this is the deployable binary compatibility check.

### Live startup

For the exact deployed commit:

1. Verify the deployment provider reports success and identifies the expected
   commit.
2. Verify Pumpkin reaches `Server is now running`.
3. Verify every plugin in the current real-plugin matrix either enables
   cleanly or has a documented configuration-only reason not to.
4. Filter deployment logs by each newly bridged event/API name. A native
   registration record must exist and there must be no matching
   `Unsupported Bukkit event type` or unimplemented-method error.

### Observable behavior

Registration proves wiring, not behavior. For events or APIs that mutate game
state, perform one focused action with a test player or purpose-built
conformance plugin and record:

- the action taken;
- the plugin callback or state change expected;
- the visible in-game or log result;
- the deployed commit and timestamp.

Do not mark that behavior verified when only compilation or listener
registration has been observed.

### Useful-test rule

Add a test only when it protects a real contract, regression, safety property,
or deployment outcome. Prefer a small, readable assertion that fails for the
original bug over broad synthetic coverage with no human-observable meaning.

## Future loader adapters

The detailed architecture, family boundaries, versioning rules, and
human-verification gates are recorded in
[`LOADER_COMPATIBILITY_ADAPTERS.md`](LOADER_COMPATIBILITY_ADAPTERS.md).

Fabric and Quilt can share a loader-adapter family. Forge and NeoForge can
share another family. These adapters will sit beside the Bukkit/Paper adapter
and reuse Pumpkin-facing bridge services, rather than mixing incompatible
loader APIs into one runtime.

The adapters will be split into these layers:

1. A loader-neutral Pumpkin bridge for players, worlds, entities, registries,
   commands, networking, scheduling, configuration, and lifecycle hooks.
2. A Bukkit/Spigot/Paper front end implemented by PatchBukkit.
3. A Fabric/Quilt front end for their loader, event, registry, networking, and
   entrypoint APIs.
4. A Forge/NeoForge front end for their mod bus, capabilities/attachments,
   registries, networking, configuration, and lifecycle APIs.

Each loader family has its own conformance suite and real-mod matrix. A loader
adapter is not complete merely because a mod JAR is discovered or its
entrypoint runs; observable game state and network behavior must match the
source API's contract.

Public loader APIs can be bridged systematically. Mods that inject Mixins,
access widened Minecraft internals, depend on mappings-specific classes, or
patch JVM bytecode require targeted internal shims, source ports, or a
dedicated JVM compatibility runtime. They are tracked separately and are not
counted as public API compatibility.

Work begins on these loader families after the Bukkit/Spigot/Paper release
gates pass, so all adapters can reuse a tested bridge kernel.
