Implementation Status
=====================

Implementation Coverage
-----------------------

.. needpie:: Requirement Implementation Coverage
   :labels: Has Implementation, No Implementation
   :legend:
   :colors: #27ae60, #e74c3c

   type == 'req' and len(implements_back) > 0
   type == 'req' and len(implements_back) == 0

Requirements Without Implementation
------------------------------------

Functional requirements that do not yet have an ``impl`` need linking to
them.

.. needtable::
   :filter: type == 'req' and len(implements_back) == 0
   :columns: id;title;status;release
   :style: table
   :sort: id

.. note::

   For the detailed implementation-to-requirement mapping, per-module
   breakdowns, and flow diagrams, see
   :doc:`Implementation Mapping </implementation/index>`.
