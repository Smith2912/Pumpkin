# Vanilla engine foundation release

This record defines the reproducible candidate and the human acceptance gate
for the first Minecraft 26.2 mob, item-physics, conifer, and village-generation
parity batch.

## Behavioral reference

- Minecraft version: `26.2`
- Fabric Loom: `1.17.17`
- Fabric Loader used by the private reference workspace: `0.19.3`
- Gradle: `9.5.1`
- Java: Eclipse Temurin `25.0.3+9`

Loom-generated Minecraft sources are a private, disposable behavioral
reference. Do not copy them into Pumpkin, commit them, publish them, or quote
them in release evidence. Pumpkin's implementation and tests must stand on
their own.

## Included contracts

- A navigator-owned movement input is not cleared by the idle move controller.
- Every living mob shares the 26.2 ambient-sound timer. Mobs with a standard
  `entity.<type>.ambient` event discover it from Pumpkin's generated sound
  registry; state-dependent mobs may override the mapping.
- Item entities run common entity bookkeeping before physics, use current
  fluid state, and synchronize position independently from velocity.
- Pine crowns include every layer in top-to-bottom order.
- Mega-pine crowns use the calculated expanding radius, include the top layer,
  and preserve the alternating jagged edge.
- Jigsaw pieces align their ground plane rather than their template floor and
  preserve the inclusive expansion height.
- Entities embedded in structure templates are transformed with their
  structure, assigned a stable placement identity derived from the template or
  piece plus entity ordinal, owned by exactly one chunk, persisted while
  pending, and created exactly once by the world runtime. Structure-spawned
  mobs receive common spawn finalization, and villagers request an immediate
  profession/point-of-interest scan. This covers bundled village villager,
  animal, cat, and iron-golem templates.

## Automated gates

Use one named disposable target directory on `D:` and stop to clean it if it
approaches 20 GiB. Run only these contract-focused checks during development:

1. Ambient timer and generated-registry mapping tests.
2. Item position/velocity synchronization threshold tests.
3. Pine and mega-pine crown layer/radius tests.
4. Jigsaw ground-plane and expansion-height tests.
5. Structure-entity transform, chunk ownership, stable-identity, pending-NBT,
   spawn-finalization, immediate villager-scan, and bundled villager/golem
   template tests.
6. Formatting and whitespace validation.

For the final candidate, perform one clean workspace release build and one
Docker image build. Record commands, tool versions, results, hashes, UTC
timestamps, and the final target-directory size. Delete the disposable Rust,
Gradle, Loom, and JDK reference caches after the release outcome is known.

## Human acceptance

World generation is not retroactive. Test conifers, villages, and embedded
entities in a new disposable world or chunks that have never been generated.
Do not judge this batch from the already-generated broken village.

1. In a fresh plains area, watch at least two cows, pigs, chickens, or sheep
   for two minutes. They must wander independently and emit audible ambient
   sounds without player interaction.
2. Drop an axe and another item over land and over water. Their rendered
   positions must follow their server positions, sink/float according to the
   current fluid state, and be collectible where they appear without walking
   underneath a stale visual.
3. Generate or locate a new plains village. Houses and roads must use the
   correct ground plane instead of being shifted into the terrain.
4. Confirm the new village contains the entities selected by its jigsaw pools,
   including villagers and any selected animal, cat, or iron-golem template.
   Move away, return, restart once, and confirm they do not duplicate.
5. Generate new pine and mega-pine trees. Crowns must include their top layer
   and expand down the trunk; bare log towers with tiny or absent crowns fail
   this gate.
6. Place and remove water sources on flat ground and at an edge. Confirm
   downward flow, horizontal spread, draining, and item buoyancy remain stable.
7. Confirm logs contain no panic, duplicate structure entity, invalid entity
   NBT, chunk-generation failure, disconnect, or restart from this workflow.

## Deployment record

- Rollback Pumpkin SHA: `b44087fc81f4372f90a3afe184fd1a4be2b35891`
- Candidate Pumpkin SHA:
- Loom/Gradle/JDK reference versions: `1.17.17` / `9.5.1` /
  Eclipse Temurin `25.0.3+9`
- Focused test result: `18 passed, 0 failed`; formatting and `git diff --check`
  passed
- Clean release-build result: passed with Rust/Cargo `1.97.1`; started
  `2026-07-25T05:47:36.0718219Z`, completed
  `2026-07-25T06:09:06.1422500Z` in 21m 29s; clean target size 2.75 GiB;
  `pumpkin.exe` 73,068,544 bytes, SHA-256
  `413dff6fbebf04148d7e4464386682a2732d0a1de55bc15f95ff1431366e46a2`
- Docker image digest:
- Railway deployment ID:
- Docker build started/completed UTC:
- Startup completed UTC:
- Startup excerpt:
- Fresh-world seed and tested coordinates:
- Human acceptance result:
- Log review result:
- Rollback decision:

Roll back to the recorded baseline if startup fails, item interaction regresses,
chunk generation becomes unstable, structure entities duplicate, or the fresh
world workflow fails.

## Explicitly deferred

This batch does not claim full vanilla AI for every entity, complete village
POI/social behavior, raid or iron-golem population rules outside structure
templates, retroactive repair of generated chunks, exact parity for every tree
placer, all fluid edge cases, or full terrain/noise parity. Those remain
subsequent evidence-driven engine batches.
