API Reference
=============

The AirGap Transfer crate exposes its internal modules through a library
target (``src/lib.rs``), enabling full API documentation via ``rustdoc``.

Generating API Docs
-------------------

.. code-block:: bash

   cargo doc --document-private-items --open

This generates browsable HTML documentation for all public types, traits,
and functions across every module:

- **chunker** — Streaming chunk creation and reconstruction (tar format)
- **commands** — Pack, unpack, and list command implementations
- **error** — Error types (``AirgapError``) and result aliases
- **manifest** — JSON manifest for metadata and state persistence
- **progress** — Progress bar and byte formatting utilities
- **prompt** — Interactive user prompts for USB swapping
- **usb** — USB drive detection and capacity checks (platform-specific)
- **verifier** — Pluggable hash verification (``HashAlgorithm`` trait)

For the requirement-to-module traceability mapping, see the
:doc:`Implementation Mapping </implementation/index>` section.
