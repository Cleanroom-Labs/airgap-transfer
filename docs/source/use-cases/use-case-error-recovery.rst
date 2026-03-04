.. include:: ../../spec-docs/source/use-cases/use-case-error-recovery.rst

Linked Requirements
-------------------

.. needtable::
   :filter: type in ['req', 'nfreq'] and 'UC-TRANSFER-006' in specifies_back
   :columns: id;title;status;release;verified_by
   :style: datatables
   :sort: id

Traceability
------------

.. needflow::
   :root_id: UC-TRANSFER-006
   :root_direction: outgoing
   :root_depth: 2
   :link_types: links, specifies, verified_by

Tests
-----

Integration Tests
~~~~~~~~~~~~~~~~~

.. needtable::
   :filter: type == 'test' and 'UC-TRANSFER-006' in verified_by_back and "ci-result" not in tags
   :columns: id;title;status;tests
   :style: datatables
   :sort: id

Unit Tests of Linked Requirements
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. needtable::
   :filter: type == 'test' and 'UC-TRANSFER-006' not in verified_by_back and "ci-result" not in tags and ('FR-TRANSFER-024' in verified_by_back or 'FR-TRANSFER-025' in verified_by_back or 'FR-TRANSFER-027' in verified_by_back or 'FR-TRANSFER-035' in verified_by_back or 'FR-TRANSFER-036' in verified_by_back or 'FR-TRANSFER-037' in verified_by_back)
   :columns: id;title;status;tests
   :style: datatables
   :sort: id
