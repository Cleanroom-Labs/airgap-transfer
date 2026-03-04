.. include:: ../../spec-docs/source/use-cases/use-case-verify-integrity.rst

Linked Requirements
-------------------

.. needtable::
   :filter: type in ['req', 'nfreq'] and 'UC-TRANSFER-005' in specifies_back
   :columns: id;title;status;release;verified_by
   :style: datatables
   :sort: id

Traceability
------------

.. needflow::
   :root_id: UC-TRANSFER-005
   :root_direction: outgoing
   :root_depth: 2
   :link_types: links, specifies, verified_by

Tests
-----

Integration Tests
~~~~~~~~~~~~~~~~~

.. needtable::
   :filter: type == 'test' and 'UC-TRANSFER-005' in verified_by_back and "ci-result" not in tags
   :columns: id;title;status;tests
   :style: datatables
   :sort: id

Unit Tests of Linked Requirements
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. needtable::
   :filter: type == 'test' and 'UC-TRANSFER-005' not in verified_by_back and "ci-result" not in tags and ('FR-TRANSFER-016' in verified_by_back or 'FR-TRANSFER-017' in verified_by_back or 'FR-TRANSFER-018' in verified_by_back or 'FR-TRANSFER-019' in verified_by_back or 'FR-TRANSFER-022' in verified_by_back or 'FR-TRANSFER-057' in verified_by_back)
   :columns: id;title;status;tests
   :style: datatables
   :sort: id
