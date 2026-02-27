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

   type == 'req' and len(tests_back) > 0
   type == 'req' and len(tests_back) == 0

.. needpie:: NFR Test Coverage
   :labels: Has Tests, No Tests
   :legend:
   :colors: #27ae60, #e74c3c

   type == 'nfreq' and len(tests_back) > 0
   type == 'nfreq' and len(tests_back) == 0

.. needpie:: Requirements by Release
   :labels: v1.0, v1.1, v1.2
   :legend:
   :colors: #2980b9, #f39c12, #9b59b6

   type in ['req', 'nfreq'] and release == 'v1.0'
   type in ['req', 'nfreq'] and release == 'v1.1'
   type in ['req', 'nfreq'] and release == 'v1.2'

