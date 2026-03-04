Coverage Gaps
=============

v1.0 requirements and non-functional requirements that are missing test
cases, implementation mapping, or both.

.. note::

   When all v1.0 requirements have full test and implementation coverage,
   the tables below will be empty — this is the expected healthy state.

Functional Requirements Without Tests
--------------------------------------

.. needtable::
   :filter: type == 'req' and len(verified_by) == 0 and release == 'v1.0'
   :columns: id;title;status;release
   :style: datatables
   :sort: id

Non-Functional Requirements Without Tests
-------------------------------------------

.. needtable::
   :filter: type == 'nfreq' and len(verified_by) == 0 and release == 'v1.0'
   :columns: id;title;status;release
   :style: datatables
   :sort: id

Requirements Without Implementation Mapping
---------------------------------------------

.. note::

   This section will become meaningful after ``impl`` needs are added in
   Phase 3.

.. needtable::
   :filter: type == 'req' and len(realized_by) == 0 and release == 'v1.0'
   :columns: id;title;status;release
   :style: datatables
   :sort: id
