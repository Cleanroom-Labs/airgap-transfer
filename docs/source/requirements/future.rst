.. include:: ../../spec-docs/source/requirements/future.rst

Future Release Overview
-----------------------

.. needtable::
   :filter: type in ['req', 'nfreq'] and release != 'v1.0'
   :columns: id;title;release;verified_by
   :style: datatables
   :sort: release

.. needpie:: Future-Release Test Coverage
   :labels: Has Tests, No Tests
   :legend:
   :colors: #27ae60, #e74c3c

   type in ['req', 'nfreq'] and release != 'v1.0' and len(verified_by) > 0
   type in ['req', 'nfreq'] and release != 'v1.0' and len(verified_by) == 0
