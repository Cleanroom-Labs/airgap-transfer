CLI Infrastructure
==================

``src/main.rs``, ``src/error.rs``, ``src/progress.rs``,
``src/prompt.rs`` — Shared CLI infrastructure.

CLI Flags
---------

.. impl:: CLI — Pack Command
   :id: IMPL-CLI-001
   :status: implemented
   :tags: cli, clap
   :release: v1.0

   ``clap`` derive-based CLI with ``pack``, ``unpack``, and ``list``
   subcommands.

.. impl:: CLI — Dry Run
   :id: IMPL-CLI-002
   :status: implemented
   :tags: cli, dry-run
   :release: v1.0

   ``--dry-run`` prints what would happen without writing any chunks.

.. impl:: CLI — No-Verify
   :id: IMPL-CLI-003
   :status: implemented
   :tags: cli, verification
   :release: v1.0

   ``--no-verify`` skips checksum verification. Verification is on by
   default.

.. impl:: CLI — Chunk Size Flag
   :id: IMPL-CLI-004
   :status: implemented
   :tags: cli, chunk-size
   :release: v1.0

   ``--chunk-size SIZE`` accepts human-readable values (e.g., ``1G``).

.. impl:: CLI — Verbose
   :id: IMPL-CLI-005
   :status: implemented
   :tags: cli, verbose
   :release: v1.0

   ``--verbose`` enables detailed output.

.. impl:: CLI — Force
   :id: IMPL-CLI-006
   :status: implemented
   :tags: cli, force
   :release: v1.0

   ``--force`` overwrites existing manifests without prompting.

Error Handling
--------------

.. impl:: Error — Insufficient Space
   :id: IMPL-ERROR-001
   :status: implemented
   :tags: error, usb, space
   :release: v1.0

   Pre-pack space check compares available bytes against chunk size
   and reports a clear error if insufficient.

.. impl:: Error — Missing Chunks
   :id: IMPL-ERROR-002
   :status: implemented
   :tags: error, unpack
   :release: v1.0

   ``AirgapError`` variants provide structured errors for missing
   chunks, verification failures, and I/O problems.

.. impl:: Error — Clear Messages
   :id: IMPL-ERROR-003
   :status: implemented
   :tags: error, ux
   :release: v1.0

   ``thiserror`` derive macros produce human-readable error messages
   with context (file paths, chunk indices).

Safety
------

.. impl:: Safety — Validate Paths
   :id: IMPL-SAFETY-001
   :status: implemented
   :tags: safety, validation
   :release: v1.0

   Source and destination paths are validated for existence and
   writability before operations begin.

.. impl:: Safety — Atomic Operations
   :id: IMPL-SAFETY-002
   :status: implemented
   :tags: safety, atomic
   :release: v1.0

   Manifest is saved after each chunk completion so that interrupted
   operations can resume.

Build
-----

.. impl:: Build — Offline Dependencies
   :id: IMPL-BUILD-001
   :status: implemented
   :tags: build, offline
   :release: v1.0

   ``cargo vendor`` and ``cargo build --release --offline`` support
   fully air-gapped builds.

.. impl:: Build — Static Binary
   :id: IMPL-BUILD-002
   :status: implemented
   :tags: build, binary
   :release: v1.0

   Release build produces a single static binary with no runtime
   dependencies.

Resume
------

.. impl:: Resume — Interrupted Pack
   :id: IMPL-RESUME-001
   :status: implemented
   :tags: resume, pack
   :release: v1.0

   ``--resume`` detects existing manifest, validates compatibility,
   and resumes from the first incomplete chunk.

.. impl:: Resume — Interrupted Unpack
   :id: IMPL-RESUME-002
   :status: implemented
   :tags: resume, unpack
   :release: v1.0

   Unpack uses manifest chunk status to skip already-extracted chunks.

.. impl:: Manifest — Final Checksum
   :id: IMPL-VERIFY-FINAL
   :status: implemented
   :tags: verifier, checksum
   :release: v1.0

   After all chunks are unpacked, a final integrity check verifies the
   reassembled output.
