Implementation Mapping
======================

This section maps Rust source modules to the requirements they implement.
Each ``impl`` need links to the requirements it satisfies using the
``:implements:`` relation, enabling bidirectional traceability from
code modules to formal specifications.

Module Overview
---------------

.. needtable::
   :types: impl
   :columns: id;title;status;release;implements
   :style: datatables
   :sort: id

Requirement → Implementation Flow
----------------------------------

.. needflow::
   :filter: type in ['req', 'impl']
   :link_types: implements

.. toctree::
   :maxdepth: 1
   :caption: Modules

   chunker
   verifier
   manifest
   usb
   commands
   cli
