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

## Future loader adapters

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
