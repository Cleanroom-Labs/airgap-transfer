Project Health
==============

At-a-glance project health.

.. needpie:: Needs by Type
   :labels: Functional Req, Non-Functional Req, Test Case, Use Case
   :legend:
   :colors: #FEDCD2, #DF744A, #84B39D, #BFD8D2

   type == 'req'
   type == 'nfreq'
   type == 'test'
   type == 'usecase'

.. needpie:: Requirement Test Coverage
   :labels: Has Tests, No Tests
   :legend:
   :colors: #27ae60, #e74c3c

   type == 'req' and len(verified_by) > 0
   type == 'req' and len(verified_by) == 0

.. needpie:: NFR Test Coverage
   :labels: Has Tests, No Tests
   :legend:
   :colors: #27ae60, #e74c3c

   type == 'nfreq' and len(verified_by) > 0
   type == 'nfreq' and len(verified_by) == 0

.. needpie:: v1.0 Requirement Test Coverage
   :labels: Has Tests, No Tests
   :legend:
   :colors: #27ae60, #e74c3c

   type == 'req' and release == 'v1.0' and len(verified_by) > 0
   type == 'req' and release == 'v1.0' and len(verified_by) == 0

.. needpie:: Requirements by Release
   :labels: v1.0, v1.1, v1.2
   :legend:
   :colors: #2980b9, #f39c12, #9b59b6

   type in ['req', 'nfreq'] and release == 'v1.0'
   type in ['req', 'nfreq'] and release == 'v1.1'
   type in ['req', 'nfreq'] and release == 'v1.2'

Specification Coverage
----------------------

Requirements Without Use Case Coverage
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

v1.0 functional requirements not referenced by any use case.  These are
typically cross-cutting infrastructure (CLI command/flag definitions,
deployment/build concerns) or verification details implicit in all
operations.

.. needtable::
   :filter: type == 'req' and len(links_back) == 0 and release == 'v1.0'
   :columns: id;title;status;tags;release
   :style: datatables
   :sort: id

Tests Without Requirement Links
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Test cases that do not link to any requirement via the ``:tests:`` field.

.. needtable::
   :filter: type == 'test' and len(tests) == 0 and "ci-result" not in tags
   :columns: id;title;status;release
   :style: datatables
   :sort: id

v1.0 Requirements Without Test Coverage
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. needtable::
   :filter: type in ['req', 'nfreq'] and len(verified_by) == 0 and release == 'v1.0'
   :columns: id;title;status;release
   :style: datatables
   :sort: id

