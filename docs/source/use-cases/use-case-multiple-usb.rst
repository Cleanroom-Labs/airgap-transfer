.. include:: ../../spec-docs/source/use-cases/use-case-multiple-usb.rst

Linked Requirements
-------------------

.. needtable::
   :filter: type in ['req', 'nfreq'] and 'UC-TRANSFER-003' in specifies_back
   :columns: id;title;status;release;verified_by
   :style: datatables
   :sort: id

Traceability
------------

.. needflow::
   :root_id: UC-TRANSFER-003
   :root_direction: outgoing
   :root_depth: 2
   :link_types: links, specifies, verified_by

Tests
-----

Integration Tests
~~~~~~~~~~~~~~~~~

.. needtable::
   :filter: type == 'test' and 'UC-TRANSFER-003' in verified_by_back and "ci-result" not in tags
   :columns: id;title;status;tests
   :style: datatables
   :sort: id

Unit Tests of Linked Requirements
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. needtable::
   :filter: type == 'test' and 'UC-TRANSFER-003' not in verified_by_back and "ci-result" not in tags and ('FR-TRANSFER-002' in verified_by_back or 'FR-TRANSFER-006' in verified_by_back or 'FR-TRANSFER-008' in verified_by_back or 'FR-TRANSFER-012' in verified_by_back or 'FR-TRANSFER-013' in verified_by_back or 'FR-TRANSFER-024' in verified_by_back or 'FR-TRANSFER-025' in verified_by_back or 'FR-TRANSFER-026' in verified_by_back)
   :columns: id;title;status;tests
   :style: datatables
   :sort: id
