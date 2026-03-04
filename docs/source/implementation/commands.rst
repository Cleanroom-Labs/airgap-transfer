Command Modules
===============

``src/commands/pack.rs``, ``src/commands/unpack.rs``,
``src/commands/list.rs`` — CLI command handlers.

Pack Command
------------

.. impl:: Pack Command — CLI Entry Point
   :id: IMPL-PACK-001
   :status: implemented
   :tags: pack, cli, command
   :release: v1.0

   ``pack::run`` orchestrates the pack workflow: argument validation,
   manifest creation, chunk size selection, space checking, progress
   display, and streaming pack via the chunker.

.. impl:: Pack — Manual Chunk Size
   :id: IMPL-PACK-002
   :status: implemented
   :tags: pack, chunk-size
   :release: v1.0

   The ``--chunk-size`` flag accepts a human-readable size (e.g.,
   ``1G``, ``500M``) that overrides auto-detection.

.. impl:: Pack — Progress Display
   :id: IMPL-PACK-003
   :status: implemented
   :tags: pack, progress, ux
   :release: v1.0

   Real-time progress bar via ``indicatif`` showing bytes written,
   transfer rate, and ETA.

.. impl:: Pack — USB Swap Prompt
   :id: IMPL-PACK-004
   :status: implemented
   :tags: pack, usb, prompt
   :release: v1.0

   When available space is insufficient, the per-chunk callback
   prompts the user to swap USB drives before continuing.

.. impl:: Pack — Overwrite Protection
   :id: IMPL-PACK-005
   :status: implemented
   :tags: pack, safety
   :release: v1.0

   Existing manifest detection: refuses to overwrite without
   ``--force``, offers ``--resume`` for continuation.

Unpack Command
--------------

.. impl:: Unpack Command — CLI Entry Point
   :id: IMPL-UNPACK-001
   :status: implemented
   :tags: unpack, cli, command
   :release: v1.0

   ``unpack::run`` orchestrates unpack: manifest loading, chunk
   verification, tar extraction, and final checksum validation.

.. impl:: Unpack — Validate Completeness
   :id: IMPL-UNPACK-002
   :status: implemented
   :tags: unpack, validation
   :release: v1.0

   Before extraction, the manifest is loaded and each chunk file is
   checked for existence — missing chunks are reported.

.. impl:: Unpack — Resume Partial Unpack
   :id: IMPL-UNPACK-003
   :status: implemented
   :tags: unpack, resume
   :release: v1.0

   Unpack resumes from the first incomplete chunk by skipping already-
   extracted content.

.. impl:: Unpack — Delete Chunks
   :id: IMPL-UNPACK-004
   :status: implemented
   :tags: unpack, cleanup
   :release: v1.0

   After successful extraction, chunks are deleted by default.
   ``--keep-chunks`` preserves them.

.. impl:: Unpack — Progress Display
   :id: IMPL-UNPACK-005
   :status: implemented
   :tags: unpack, progress, ux
   :release: v1.0

   Progress bar shows extraction progress per chunk and overall.

List Command
------------

.. impl:: List Command — CLI Entry Point
   :id: IMPL-LIST-001
   :status: implemented
   :tags: list, cli, command
   :release: v1.0

   ``list::run`` loads the manifest and displays chunk inventory in
   a formatted table.

.. impl:: List — Display Inventory
   :id: IMPL-LIST-002
   :status: implemented
   :tags: list, display
   :release: v1.0

   Shows all chunk filenames, sizes, and statuses from the manifest.

.. impl:: List — Show Sizes and Status
   :id: IMPL-LIST-003
   :status: implemented
   :tags: list, display
   :release: v1.0

   Each chunk shows its byte size and completion status (pending,
   completed, failed).

.. impl:: List — Identify Missing
   :id: IMPL-LIST-004
   :status: implemented
   :tags: list, validation
   :release: v1.0

   Chunks not present on disk are flagged as missing in the output.

.. impl:: List — Estimated Total Size
   :id: IMPL-LIST-005
   :status: implemented
   :tags: list, display
   :release: v1.0

   Displays the total transfer size from the manifest header.

.. impl:: List — Verify Flag
   :id: IMPL-LIST-006
   :status: implemented
   :tags: list, verification
   :release: v1.0

   ``list --verify`` checks chunk checksums and reports integrity
   status alongside inventory.
