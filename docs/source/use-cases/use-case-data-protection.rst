.. include:: ../../spec-docs/source/use-cases/use-case-data-protection.rst

Linked Requirements
-------------------

.. needtable::
   :filter: type in ['req', 'nfreq'] and 'UC-TRANSFER-007' in specifies_back
   :columns: id;title;status;release;verified_by
   :style: datatables
   :sort: id

Traceability
------------

.. needflow::
   :root_id: UC-TRANSFER-007
   :root_direction: outgoing
   :root_depth: 2
   :link_types: links, specifies, verified_by

Tests
-----

Integration Tests
~~~~~~~~~~~~~~~~~

.. needtable::
   :filter: type == 'test' and 'UC-TRANSFER-007' in verified_by_back and "ci-result" not in tags
   :columns: id;title;status;tests
   :style: datatables
   :sort: id

Unit Tests of Linked Requirements
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. needtable::
   :filter: type == 'test' and 'UC-TRANSFER-007' not in verified_by_back and "ci-result" not in tags and ('FR-TRANSFER-038' in verified_by_back or 'FR-TRANSFER-039' in verified_by_back or 'FR-TRANSFER-040' in verified_by_back or 'FR-TRANSFER-041' in verified_by_back or 'FR-TRANSFER-056' in verified_by_back)
   :columns: id;title;status;tests
   :style: datatables
   :sort: id
