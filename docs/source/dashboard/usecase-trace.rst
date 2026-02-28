Use Case Traceability
=====================

All Use Cases
-------------

.. needtable::
   :types: usecase
   :columns: id;title;status;links
   :style: datatables
   :sort: id

Use Case → Requirement Flows
-----------------------------

UC-TRANSFER-001 — Large File Transfer
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. needflow::
   :filter: id == 'UC-TRANSFER-001' or (type == 'req' and 'UC-TRANSFER-001' in links_back)
   :link_types: links

UC-TRANSFER-002 — Large Directory Transfer
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. needflow::
   :filter: id == 'UC-TRANSFER-002' or (type == 'req' and 'UC-TRANSFER-002' in links_back)
   :link_types: links

UC-TRANSFER-003 — Multi-USB Dataset Transfer
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. needflow::
   :filter: id == 'UC-TRANSFER-003' or (type == 'req' and 'UC-TRANSFER-003' in links_back)
   :link_types: links

UC-TRANSFER-004 — SBOM Transfer
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. needflow::
   :filter: id == 'UC-TRANSFER-004' or (type == 'req' and 'UC-TRANSFER-004' in links_back)
   :link_types: links

UC-TRANSFER-005 — Verify Transfer Integrity
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. needflow::
   :filter: id == 'UC-TRANSFER-005' or (type == 'req' and 'UC-TRANSFER-005' in links_back)
   :link_types: links

UC-TRANSFER-006 — Recover from Transfer Failure
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. needflow::
   :filter: id == 'UC-TRANSFER-006' or (type == 'req' and 'UC-TRANSFER-006' in links_back)
   :link_types: links

UC-TRANSFER-007 — Protect Against Data Loss
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. needflow::
   :filter: id == 'UC-TRANSFER-007' or (type == 'req' and 'UC-TRANSFER-007' in links_back)
   :link_types: links

Requirements Without Use Case Coverage
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

v1.0 functional requirements not referenced by any use case.  These are
typically cross-cutting infrastructure (CLI command/flag definitions,
deployment/build concerns) or verification details implicit in all
operations.

.. needtable::
   :filter: type == 'req' and len(links_back) == 0 and release == 'v1.0'
   :columns: id;title;status;tags;release
   :style: datatables
   :sort: id
