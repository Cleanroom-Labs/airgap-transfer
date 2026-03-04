Future Use Cases
================

Use cases planned for post-MVP releases.

.. include:: ../../spec-docs/source/use-cases/use-case-sbom-transfer.rst

Linked Requirements
-------------------

.. needtable::
   :filter: type in ['req', 'nfreq'] and 'UC-TRANSFER-004' in specifies_back
   :columns: id;title;status;release;verified_by
   :style: datatables
   :sort: id

Traceability
------------

.. needflow::
   :root_id: UC-TRANSFER-004
   :root_direction: outgoing
   :root_depth: 2
   :link_types: links, specifies, verified_by

Future Use Case Overview
------------------------

.. needtable::
   :filter: type == 'usecase' and release != 'v1.0'
   :columns: id;title;release;status;specifies
   :style: datatables
   :sort: release

.. needpie:: Future Use Case Test Coverage
   :labels: Has Tests, No Tests
   :legend:
   :colors: #27ae60, #e74c3c

   type == 'usecase' and release != 'v1.0' and len(verified_by) > 0
   type == 'usecase' and release != 'v1.0' and len(verified_by) == 0
