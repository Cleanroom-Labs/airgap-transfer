Implementation Status
=====================

Implementation Coverage
-----------------------

.. needpie:: Requirement Implementation Coverage
   :labels: Has Implementation, No Implementation
   :legend:
   :colors: #27ae60, #e74c3c

   type == 'req' and len(realized_by) > 0
   type == 'req' and len(realized_by) == 0

Requirements Without Implementation
------------------------------------

Functional requirements that do not yet have an ``impl`` need linking to
them.

.. note::

   Future-release requirements (v1.1+) are expected to appear here.
   Implementation mapping is added per-release as development begins.

.. needtable::
   :filter: type == 'req' and len(realized_by) == 0 and release == 'v1.0'
   :columns: id;title;status;release
   :style: datatables
   :sort: id

.. note::

   For the detailed implementation-to-requirement mapping, per-module
   breakdowns, and flow diagrams, see
   :doc:`Implementation Mapping </implementation/index>`.
