# Public-server foundation release

This record defines the reproducible candidate and the human acceptance gate
for the metadata, permissions, and protected entity-interaction batch.

## Immutable inputs

- Rollback Pumpkin commit: `9e7f3809e9fa77c9593e803268834f99417666b7`
- PatchBukkit commit: `fcadfea17adf8ccde166b9e524f6d7029ead5a0e`
- Rust toolchain: `nightly-2026-03-05`
- Expected live matrix: LuckPerms `5.5.60` and EssentialsX official modules
  `2.22.0`
- EssentialsDiscord and DiscordLink are configuration-only exceptions until
  valid Discord credentials are supplied.

The deployable Pumpkin commit is the third commit on
`codex/public-server-foundation`. Record its full SHA from `git rev-parse HEAD`
in the deployment record; a commit cannot embed its own hash.

## Patch replay

The root `Dockerfile` checks and applies these patches, in order, to the pinned
PatchBukkit commit:

1. `docker/patchbukkit-26.2.patch`
2. `docker/patchbukkit-public-server-metadata.patch`
3. `docker/patchbukkit-public-server-interaction.patch`
4. `docker/patchbukkit-public-server-conformance.patch`

Normal-context `git apply --check` is mandatory. Do not use
`--unidiff-zero`, skip a patch, or accept a partially matching source tree.

## Automated release gates

Run these gates once from a fresh replay for the final candidate:

1. `git diff --check` on the applied PatchBukkit source.
2. PatchBukkit Java/protobuf and conformance-plugin JAR build, including
   `:patchbukkit:nativeSubscriptionConformance` to prove both Bukkit entity
   interaction variants share one native subscription.
3. Focused native entity-interaction tests.
4. Pumpkin and PatchBukkit release builds against the same local Pumpkin
   crates.
5. Root Docker image build.

Record the commands, tool versions, result, artifact hashes, and UTC completion
time. Delete the batch-specific build cache after the release outcome is known.

## Human acceptance

Use one non-permitted player without reconnecting between permission changes:

1. Confirm EssentialsAntiBuild denies placing, breaking, protected item use,
   and right-clicking an item frame or armor stand.
2. Grant `essentials.build` through LuckPerms and immediately repeat the four
   actions. All must work.
3. Revoke `essentials.build` and immediately repeat them. Denial must return.
4. Recheck `/gamemode survival`, Essentials teleport and flight, item
   metadata, inventory commands, fire/fluid/weather protection, and LuckPerms
   permission checks.
5. Exercise main-hand, off-hand, and positional entity interaction. Cancelling
   the Bukkit event must prevent the native use. Attacking the entity must not
   be reported as a right-click.
6. Confirm the workflow logs contain no `hasMetadata`, Java
   global-reference, panic, disconnect, or restart errors.
7. Confirm startup contains exactly one native registration for
   `PlayerInteractEntityEvent` and no unsupported registration for either
   entity-interaction class.

## Deployment record

Complete this record for the one Railway deployment:

- Candidate Pumpkin SHA:
- Patch artifact SHA-256 values:
- Java/test-plugin artifact SHA-256 values:
- Pumpkin and PatchBukkit native artifact SHA-256 values:
- Docker image digest:
- Railway deployment ID:
- Build started/completed UTC:
- Startup completed UTC:
- Startup excerpt:
- Automated gates:
- Human acceptance results:
- Log review result:
- Rollback decision:

Roll back to `9e7f3809e9fa77c9593e803268834f99417666b7` if startup fails,
the server becomes unstable, or the primary permission/protection workflow
fails.

## Explicitly deferred

This batch does not claim persistent-data containers, entity damage, PvP,
explosions, arbitrary entity wrappers, Forge, NeoForge, Fabric, Quilt, or full
Bukkit/Spigot/Paper coverage. Metadata is process-local and intentionally does
not survive a server restart.
