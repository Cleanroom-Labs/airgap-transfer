Chunker Module
==============

``src/chunker.rs`` — Streaming chunk creation and reconstruction.

.. impl:: Chunker — File Splitting
   :id: IMPL-CHUNKER-001
   :status: implemented
   :tags: chunker, pack, streaming
   :release: v1.0
   :implements: FR-TRANSFER-001

   Streams source files and directories into fixed-size ``chunk_XXX.tar``
   archives. Uses an 8 KiB read buffer to keep memory well under the
   100 MB budget.

.. impl:: Chunker — Stream to USB
   :id: IMPL-CHUNKER-002
   :status: implemented
   :tags: chunker, streaming
   :release: v1.0
   :implements: FR-TRANSFER-005

   Data is written directly to the destination (USB) path without
   intermediate temp files — a streaming architecture.

.. impl:: Chunker — Reconstruction
   :id: IMPL-CHUNKER-003
   :status: implemented
   :tags: chunker, unpack, streaming
   :release: v1.0
   :implements: FR-TRANSFER-009

   ``untar_chunks`` extracts files from a series of tar chunks back into
   the original directory structure.

.. impl:: Chunker — Place Files in Destination
   :id: IMPL-CHUNKER-004
   :status: implemented
   :tags: chunker, unpack
   :release: v1.0
   :implements: FR-TRANSFER-011

   Reconstructed files are placed in the user-specified destination
   directory, recreating the original directory hierarchy.

.. impl:: Chunker — Resume Pack
   :id: IMPL-CHUNKER-005
   :status: implemented
   :tags: chunker, resume
   :release: v1.0
   :implements: FR-TRANSFER-026

   ``pack_to_chunks_with_callback`` accepts a ``resume_from`` index in
   its ``PackConfig``. Chunks before that index are simulated (sizes
   tracked but not written), enabling efficient resume without
   re-processing.
