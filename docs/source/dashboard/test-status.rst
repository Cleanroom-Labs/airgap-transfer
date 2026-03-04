Test Status
===========

Tests Requiring Attention
-------------------------

.. note::

   Tests with status other than ``approved`` need review.
   When CI results are available (via ``make full``), failures appear here.

.. needtable::
   :filter: type == 'test' and status != 'approved'
   :columns: id;title;status;release;tests
   :style: datatables
   :sort: id

Test Execution Results
----------------------

.. note::

   This table is populated by ``make full``, which runs the Rust test
   suite and imports results.  If the table is empty, the docs were built
   with ``make html`` (no test execution).

.. needtable::
   :filter: type == 'test' and "ci-result" in tags
   :columns: id;title;status;links
   :style: datatables
   :sort: id

All Test Cases
--------------

.. needtable::
   :types: test
   :columns: id;title;status;release;tests
   :style: datatables
   :sort: id

v1.0 Test Cases
---------------

.. needtable::
   :filter: type == 'test' and release == 'v1.0'
   :columns: id;title;status;tests
   :style: datatables
   :sort: id

v1.1 Test Cases
---------------

.. note::

   Test cases for this release will be added when development begins.

.. needtable::
   :filter: type == 'test' and release == 'v1.1'
   :columns: id;title;status;tests
   :style: datatables
   :sort: id

v1.2 Test Cases
---------------

.. note::

   Test cases for this release will be added when development begins.

.. needtable::
   :filter: type == 'test' and release == 'v1.2'
   :columns: id;title;status;tests
   :style: datatables
   :sort: id
