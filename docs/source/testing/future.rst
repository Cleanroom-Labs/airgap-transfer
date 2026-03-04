Future Test Plan
================

Test cases for post-MVP releases will be tracked here as future requirements
are implemented.

Future Test Overview
--------------------

.. needtable::
   :filter: type == 'test' and release != 'v1.0'
   :columns: id;title;release;status;verified_by
   :style: datatables
   :sort: release

.. needpie:: Future Test Coverage
   :labels: Has Tests, No Tests
   :legend:
   :colors: #27ae60, #e74c3c

   type in ['req', 'nfreq'] and release != 'v1.0' and len(verified_by) > 0
   type in ['req', 'nfreq'] and release != 'v1.0' and len(verified_by) == 0
