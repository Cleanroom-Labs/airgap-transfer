Coverage Gaps
=============

This page identifies requirements and non-functional requirements that are
missing test cases, implementation mapping, or both.

Functional Requirements Without Tests
--------------------------------------

.. needtable::
   :filter: type == 'req' and len(tests_back) == 0
   :columns: id;title;status;release
   :style: table
   :sort: id

Non-Functional Requirements Without Tests
-------------------------------------------

.. needtable::
   :filter: type == 'nfreq' and len(tests_back) == 0
   :columns: id;title;status;release
   :style: table
   :sort: id

Requirements Without Implementation Mapping
---------------------------------------------

.. note::

   This section will become meaningful after ``impl`` needs are added in
   Phase 3.

.. needtable::
   :filter: type == 'req' and len(implements_back) == 0
   :columns: id;title;status;release
   :style: table
   :sort: id

Future-Release Requirements (v1.1+)
-------------------------------------

Requirements scheduled beyond v1.0 that may not yet have tests or
implementation.

.. needtable::
   :filter: type in ['req', 'nfreq'] and release != 'v1.0'
   :columns: id;title;release;tests_back
   :style: table
   :sort: release
