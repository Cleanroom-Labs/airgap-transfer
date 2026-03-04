.. include:: ../../spec-docs/source/use-cases/use-case-large-file.rst

Linked Requirements
-------------------

.. needtable::
   :filter: type in ['req', 'nfreq'] and 'UC-TRANSFER-001' in specifies_back
   :columns: id;title;status;release;verified_by
   :style: datatables
   :sort: id

Traceability
------------

.. needflow::
   :root_id: UC-TRANSFER-001
   :root_direction: outgoing
   :root_depth: 2
   :link_types: links, specifies, verified_by

Tests
-----

Integration Tests
~~~~~~~~~~~~~~~~~

.. needtable::
   :filter: type == 'test' and 'UC-TRANSFER-001' in verified_by_back and "ci-result" not in tags
   :columns: id;title;status;tests
   :style: datatables
   :sort: id

Unit Tests of Linked Requirements
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. needtable::
   :filter: type == 'test' and 'UC-TRANSFER-001' not in verified_by_back and "ci-result" not in tags and ('FR-TRANSFER-001' in verified_by_back or 'FR-TRANSFER-003' in verified_by_back or 'FR-TRANSFER-005' in verified_by_back or 'FR-TRANSFER-007' in verified_by_back or 'FR-TRANSFER-009' in verified_by_back or 'FR-TRANSFER-010' in verified_by_back or 'FR-TRANSFER-020' in verified_by_back or 'FR-TRANSFER-031' in verified_by_back)
   :columns: id;title;status;tests
   :style: datatables
   :sort: id
