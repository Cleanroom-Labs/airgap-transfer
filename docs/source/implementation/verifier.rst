Verifier Module
===============

``src/verifier.rs`` — Pluggable hash verification with the
``HashAlgorithm`` trait.

.. impl:: Verifier — Generate Checksums
   :id: IMPL-VERIFIER-001
   :status: implemented
   :tags: verifier, hash, checksum
   :release: v1.0
   :implements: FR-TRANSFER-003, FR-TRANSFER-020

   ``HashAlgorithm`` trait with ``create_writer`` and ``verify_file``
   methods. SHA-256 is the default backend.

.. impl:: Verifier — Verify During Unpack
   :id: IMPL-VERIFIER-002
   :status: implemented
   :tags: verifier, unpack, checksum
   :release: v1.0
   :implements: FR-TRANSFER-010, FR-TRANSFER-021

   Chunk checksums are verified against manifest values before
   unpacking. Corrupted chunks are detected and reported.

.. impl:: Verifier — Detect Corruption
   :id: IMPL-VERIFIER-003
   :status: implemented
   :tags: verifier, checksum, error
   :release: v1.0
   :implements: FR-TRANSFER-022

   Mismatched checksums produce a clear error identifying the
   corrupted chunk index and expected vs. actual hash.

.. impl:: Verifier — Configurable Algorithm
   :id: IMPL-VERIFIER-004
   :status: implemented
   :tags: verifier, hash, algorithm
   :release: v1.0
   :implements: FR-TRANSFER-045

   The ``--hash-algorithm`` CLI flag selects the backend. Currently
   SHA-256; the trait design supports adding BLAKE3 or other algorithms.

.. impl:: Verifier — Pluggable Backend
   :id: IMPL-VERIFIER-005
   :status: implemented
   :tags: verifier, trait, extensibility
   :release: v1.0
   :implements: FR-TRANSFER-047

   The ``HashAlgorithm`` trait defines a pluggable interface: any type
   implementing ``create_writer`` and ``verify_file`` can serve as a
   hash backend.
