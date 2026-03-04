USB Module
==========

``src/usb.rs`` — USB detection and capacity checks (platform-specific).

.. impl:: USB — Auto-Detect Capacity
   :id: IMPL-USB-001
   :status: implemented
   :tags: usb, capacity, detection
   :release: v1.0

   ``get_available_space`` queries available bytes on the destination
   volume. Platform-specific: macOS (``/Volumes/*``), Linux
   (``/media/$USER/*``).

.. impl:: USB — Sync Safely
   :id: IMPL-USB-002
   :status: implemented
   :tags: usb, sync, safety
   :release: v1.0

   ``sync_filesystem`` calls the platform ``sync`` syscall to flush
   buffers before the user removes the drive.
