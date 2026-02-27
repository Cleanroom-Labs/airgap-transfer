Manifest Module
===============

``src/manifest.rs`` — JSON manifest for transfer metadata and state
persistence.

.. impl:: Manifest — Create Manifest File
   :id: IMPL-MANIFEST-001
   :status: implemented
   :tags: manifest, json, metadata
   :release: v1.0
   :implements: FR-TRANSFER-004

   ``Manifest::new`` creates a manifest tracking source path, chunk
   count, sizes, checksums, and algorithm. Serialized to
   ``airgap-transfer-manifest.json``.

.. impl:: Manifest — Operation State
   :id: IMPL-MANIFEST-002
   :status: implemented
   :tags: manifest, state, resume
   :release: v1.0
   :implements: FR-TRANSFER-024, FR-TRANSFER-025

   ``update_chunk`` tracks per-chunk status (pending, in-progress,
   completed, failed). ``first_incomplete_chunk`` enables resume by
   finding the first non-completed chunk.

.. impl:: Manifest — Algorithm in Manifest
   :id: IMPL-MANIFEST-003
   :status: implemented
   :tags: manifest, hash, algorithm
   :release: v1.0
   :implements: FR-TRANSFER-046

   The ``hash_algorithm`` field in the manifest records which algorithm
   was used, so the unpack side uses the matching verifier.
